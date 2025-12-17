//
//  BroadlinkRemote.swift
//  BroadlinkRemote
//
//  Created by Louis Margot on 14/11/2025.
//

import SwiftUI
import ServiceManagement
import Combine

@main
struct BroadlinkRemoteApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    var body: some Scene {
        Settings { EmptyView() }
    }
}
