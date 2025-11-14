//
//  BroadlinkConfig.swift
//  BroadlinkRemote
//
//  Created by Louis Margot on 03/11/2025.
//

import Foundation
import SwiftUI
import Combine

class BroadlinkConfig: ObservableObject {
    @AppStorage("Broadlink_host") private var storedHost: String = "192.168.1.143"
    @AppStorage("Broadlink_port") private var storedPort: Int = 6676
    @Published var host: String = "192.168.1.143"
    @Published var port: Int = 6676

    init() {
        // Keep @AppStorage and @Published in sync
        host = storedHost
        port = storedPort
    }
}

