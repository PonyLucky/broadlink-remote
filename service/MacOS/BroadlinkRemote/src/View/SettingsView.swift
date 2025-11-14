//
//  SettingsView.swift
//  BroadlinkRemote
//
//  Created by Louis Margot on 03/11/2025.
//

import SwiftUI

struct SettingsView: View {
    @ObservedObject var config: BroadlinkConfig
    var onRefresh: () -> Void
    var isReachable: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            // 1. Connectivity Info
            HStack {
                Label(isReachable ? "Lecteur connecté" : "Lecteur non joignable",
                      systemImage: isReachable ? "checkmark.circle.fill" : "xmark.octagon.fill")
                    .foregroundColor(isReachable ? .green : .red)
                Spacer()
                Button("↻ Rafraîchir") {
                    onRefresh()
                }
            }

            Divider()

            // 2. IP Configuration
            VStack(alignment: .leading) {
                Text("Adresse IP du lecteur :")
                HStack {
                    TextField("192.168.1.23", text: $config.host)
                        .textFieldStyle(RoundedBorderTextFieldStyle())
                        .frame(width: 200)
                    if isReachable {
                        Image(systemName: "checkmark.circle.fill")
                            .foregroundColor(.green)
                    } else {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundColor(.red)
                    }
                }
            }

            // 3. Timing Config
            VStack(alignment: .leading, spacing: 10) {
                Text("Fréquences de rafraîchissement :")
                    .font(.headline)
                HStack(alignment: .top) {
                    VStack(alignment: .leading) {
                        Text("Intervalle normal (s)")
                        Slider(value: $config.normalInterval, in: 1...30, step: 1)
                        Text("\(Int(config.normalInterval)) s")
                            .font(.caption)
                    }
                    VStack(alignment: .leading) {
                        Text("Intervalle rapide (s)")
                        Slider(value: $config.fastInterval, in: 0.5...10, step: 0.5)
                        Text("\(config.fastInterval, specifier: "%.1f") s")
                            .font(.caption)
                    }
                    VStack(alignment: .leading) {
                        Text("Durée du mode rapide (s)")
                        Slider(value: $config.fastDuration, in: 1...15, step: 1)
                        Text("\(Int(config.fastDuration)) s")
                            .font(.caption)
                    }
                    VStack(alignment: .leading) {
                        Text("Intervalle lent (s) — si le lecteur est injoignable")
                        Slider(value: $config.slowInterval, in: 5...120, step: 5)
                        Text("\(Int(config.slowInterval)) s")
                            .font(.caption)
                    }
                }
            }

            Spacer()
        }
        .padding(20)
        .frame(width: 420)
    }
}
