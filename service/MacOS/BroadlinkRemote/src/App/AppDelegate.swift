//
//  AppDelegate.swift
//  BroadlinkRemote
//

import Foundation
import AppKit
import SwiftUI
import Combine
import MediaPlayer
import ServiceManagement

final class AppDelegate: NSObject, NSApplicationDelegate, NSWindowDelegate {
    // Core
    var statusItem: NSStatusItem!
    let config = BroadlinkConfig()

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

        // List of devices and their actions

        // ---

        menu.addItem(.separator())
        addStartupToggle(to: menu)
        menu.addItem(.separator())
        menu.addItem(withTitle: "Quitter", action: #selector(quitApp), keyEquivalent: "q")

        menu.items.forEach { $0.target = self }
        statusItem.menu = menu
    }

    private func updateStatusIcon() {
        // https://github.com/sam4096/apple-sf-symbols-list/blob/main/allsfsymbols.txt
        let icon = NSImage(systemSymbolName: "house.circle", accessibilityDescription: nil)
        icon?.isTemplate = true
        if let button = statusItem.button {
            button.image = icon
            button.imagePosition = .imageOnly
            button.title = ""
        }
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
