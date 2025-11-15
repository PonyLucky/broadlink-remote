//
//  AppDelegate.swift
//  BroadlinkRemote
//

import Foundation
import AppKit
import SwiftUI
import Combine
import ServiceManagement

final class AppDelegate: NSObject, NSApplicationDelegate, NSWindowDelegate {
    // Core
    var statusItem: NSStatusItem!
    let config = BroadlinkConfig()

    // API/cache
    private var api: BroadlinkController?
    // controller -> device -> tree
    private var treeCache: [String: [String: BLNode]] = [:]
    private var controllers: [BLControllerInfo] = []
    private var isLoading: Bool = false

    // Preferences
    private let showDisabledItemsKey = "ShowDisabledItems"
    private var showDisabledItems: Bool {
        get { UserDefaults.standard.object(forKey: showDisabledItemsKey) as? Bool ?? true }
        set { UserDefaults.standard.set(newValue, forKey: showDisabledItemsKey) }
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        // Run as a true menu bar app: hide from Dock and Cmd+Tab
        NSApp.setActivationPolicy(.accessory)
        
        setupMenuBar()
        updateStatusIcon()
        refreshDevices()
    }

    // MARK: - Clean Quit
    func cleanQuit() {
        print("🧹 BroadlinkRemote terminated cleanly.")
    }

    @objc private func quitApp() {
        cleanQuit()
        NSApplication.shared.terminate(nil)
    }

    // MARK: - Menu Bar
    private func setupMenuBar() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.squareLength)
        setupMenu()
    }

    private func setupMenu() {
        let menu = NSMenu()

        // Refresh devices
        let refreshItem = NSMenuItem(title: isLoading ? "Refreshing…" : "Refresh devices", action: #selector(onRefreshDevices), keyEquivalent: "r")
        refreshItem.isEnabled = !isLoading
        menu.addItem(refreshItem)

        // List of devices and their actions
        buildDevicesMenu(into: menu)
        menu.addItem(.separator())
        
        // Open web UI
        let openWebItem = NSMenuItem(title: "Open Web App", action: #selector(onOpenWebApp), keyEquivalent: "o")
        menu.addItem(openWebItem)

        let settingsMenu = NSMenu(title: "Settings")
        let settingsTitle = NSMenuItem(title: "Settings", action: nil, keyEquivalent: "")
        settingsTitle.submenu = settingsMenu
        addShowDisabledToggle(to: settingsMenu)
        addStartupToggle(to: settingsMenu)
        menu.addItem(settingsTitle)

        menu.addItem(.separator())
        menu.addItem(withTitle: "Quitter", action: #selector(quitApp), keyEquivalent: "q")

        menu.items.forEach { $0.target = self }
        statusItem.menu = menu
    }

    private func updateStatusIcon() {
        // https://github.com/sam4096/apple-sf-symbols-list/blob/main/allsfsymbols.txt
        let icon = NSImage(systemSymbolName: "av.remote", accessibilityDescription: nil)
        icon?.isTemplate = true
        if let button = statusItem.button {
            button.image = icon
            button.imagePosition = .imageOnly
            button.title = ""
        }
        setupMenu()
    }

    // MARK: - Build Devices/Actions Menu
    private func buildDevicesMenu(into menu: NSMenu) {
        if isLoading {
            let it = NSMenuItem()
            it.title = "Loading devices…"
            it.isEnabled = false
            menu.addItem(it)
            return
        }
        guard !controllers.isEmpty else {
            let it = NSMenuItem()
            it.title = "No controllers/devices"
            it.isEnabled = false
            menu.addItem(it)
            return
        }
        for ctrl in controllers.sorted(by: { ($0.friendly_name ?? $0.name) < ($1.friendly_name ?? $1.name) }) {
            let ctrlItem = NSMenuItem(title: ctrl.friendly_name ?? ctrl.name, action: nil, keyEquivalent: "")
            let ctrlMenu = NSMenu(title: ctrl.friendly_name ?? ctrl.name)
            // Devices
            let devMap = treeCache[ctrl.name] ?? [:]
            if devMap.isEmpty {
                let empty = NSMenuItem(title: "No devices", action: nil, keyEquivalent: "")
                empty.isEnabled = false
                ctrlMenu.addItem(empty)
            } else {
                for (devName, rootNode) in devMap.sorted(by: { $0.key < $1.key }) {
                    let devTitle = rootNode.friendlyName ?? devName
                    let devItem = NSMenuItem(title: devTitle, action: nil, keyEquivalent: "")
                    let devMenu = NSMenu(title: devTitle)
                    let count = buildTreeMenu(into: devMenu, controller: ctrl.name, device: devName, node: rootNode)
                    if showDisabledItems || count > 0 {
                        devItem.submenu = devMenu
                        ctrlMenu.addItem(devItem)
                    }
                }
            }
            ctrlItem.submenu = ctrlMenu
            menu.addItem(ctrlItem)
        }
    }

    @discardableResult
    private func buildTreeMenu(into menu: NSMenu, controller: String, device: String, node: BLNode) -> Int {
        var addedCount = 0
        for child in node.children.sorted(by: { $0.friendlyName ?? $0.name < $1.friendlyName ?? $1.name }) {
            switch child.kind {
            case .group:
                if !showDisabledItems && child.disabled { continue }
                let title = showDisabledItems && child.disabled ? child.friendlyName ?? child.name + " (disabled)" : child.friendlyName ?? child.name
                let item = NSMenuItem(title: title, action: nil, keyEquivalent: "")
                item.isEnabled = !child.disabled
                let sub = NSMenu(title: child.friendlyName ?? child.name)
                let subCount = buildTreeMenu(into: sub, controller: controller, device: device, node: child)
                // When hiding disabled, avoid empty groups
                if showDisabledItems || subCount > 0 {
                    item.submenu = sub
                    menu.addItem(item)
                    addedCount += 1
                }
            case .command:
                if !showDisabledItems && child.disabled { continue }
                let title = showDisabledItems && child.disabled ? child.friendlyName ?? child.name + " (disabled)" : child.friendlyName ?? child.name
                let item = NSMenuItem(title: title, action: #selector(onSendCommand(_:)), keyEquivalent: "")
                item.isEnabled = !child.disabled
                item.representedObject = ["controller": controller, "device": device, "cmd": child.commandPath ?? child.name]
                menu.addItem(item)
                addedCount += 1
            }
        }
        return addedCount
    }

    // MARK: - Actions
    @objc private func onRefreshDevices() {
        refreshDevices()
    }

    @objc private func onSendCommand(_ sender: NSMenuItem) {
        guard let dict = sender.representedObject as? [String: String],
              let controller = dict["controller"], let device = dict["device"], let cmd = dict["cmd"] else { return }
        Task.detached { [weak self] in
            guard let self = self else { return }
            let ok = await self.api?.sendCommand(controller: controller, device: device, commandPath: cmd) ?? false
            DispatchQueue.main.async {
                if ok {
                    print("✅ Sent: \(controller)/\(device)/\(cmd)")
                } else {
                    print("⚠️ Failed to send: \(controller)/\(device)/\(cmd)")
                }
            }
        }
    }

    @objc private func onOpenWebApp() {
        // Build http://host:port/ and open with the default browser (Firefox if set as default)
        var comps = URLComponents()
        comps.scheme = "http"
        comps.host = config.host.trimmingCharacters(in: .whitespacesAndNewlines)
        comps.port = config.port
        comps.path = "/"
        guard let url = comps.url else {
            print("⚠️ Invalid host/port: \(config.host):\(config.port)")
            return
        }
        NSWorkspace.shared.open(url)
    }

    private func refreshDevices() {
        isLoading = true
        setupMenu()
        api = BroadlinkController(host: config.host, port: config.port)
        Task.detached { [weak self] in
            guard let self = self, let api = self.api else { return }
            let ctrls = await api.fetchControllers()
            var cache: [String: [String: BLNode]] = [:]
            for c in ctrls {
                let devs = await api.fetchDevices(controller: c.name)
                var map: [String: BLNode] = [:]
                for d in devs {
                    if let tree = await api.fetchCommandTree(controller: c.name, device: d.name) {
                        map[d.name] = tree
                    }
                }
                cache[c.name] = map
            }
            DispatchQueue.main.async {
                self.controllers = ctrls
                self.treeCache = cache
                self.isLoading = false
                self.setupMenu()
            }
        }
    }

    // MARK: - Disabled Items Toggle
    private func addShowDisabledToggle(to menu: NSMenu) {
        let item = NSMenuItem()
        item.title = "Show disabled commands/groups"
        item.state = showDisabledItems ? .on : .off
        item.action = #selector(toggleShowDisabled)
        item.target = self
        menu.addItem(item)
    }

    @objc private func toggleShowDisabled() {
        showDisabledItems.toggle()
        setupMenu()
    }

    // MARK: - Launch at Startup Toggle
    private func addStartupToggle(to menu: NSMenu) {
        let item = NSMenuItem()
        item.title = "Lancer au démarrage"
        item.state = isLoginItemEnabled ? .on : .off
        item.action = #selector(toggleLaunchAtLogin)
        item.target = self
        menu.addItem(item)
    }

    private var isLoginItemEnabled: Bool {
        if #available(macOS 13.0, *) {
            return SMAppService.mainApp.status == .enabled
        } else {
            return UserDefaults.standard.bool(forKey: "LaunchAtLoginLegacy")
        }
    }

    @objc private func toggleLaunchAtLogin() {
        if #available(macOS 13.0, *) {
            let service = SMAppService.mainApp
            do {
                if service.status == .enabled {
                    try service.unregister()
                    print("🚫 Launch at login disabled")
                } else {
                    try service.register()
                    print("✅ Launch at login enabled")
                }
            } catch {
                print("⚠️ Failed to toggle launch at login: \(error)")
            }
        } else {
            let newValue = !UserDefaults.standard.bool(forKey: "LaunchAtLoginLegacy")
            UserDefaults.standard.set(newValue, forKey: "LaunchAtLoginLegacy")
            print("🔁 Legacy toggle \(newValue)")
        }
        setupMenu()
    }
}
