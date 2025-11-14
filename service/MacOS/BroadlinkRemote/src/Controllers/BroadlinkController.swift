//
//  BroadlinkController.swift
//  BroadlinkRemote
//

import Foundation

class BroadlinkController {
    let host: String
    let port: Int
    private var activeTasks = [URLSessionTask]()

    init(host: String, port: Int) {
        self.host = host
        self.port = port
    }

    func cancelAllRequests() {
        for task in activeTasks {
            task.cancel()
        }
        activeTasks.removeAll()
    }

    private func getURL(_ endpoint: String) -> URL? {
        URL(string: "http://\(host):\(port)\(endpoint)")
    }

    private func sendRequest(_ endpoint: String, timeout: TimeInterval = 2.0) async -> (Data?, Bool) {
        guard let url = getURL(endpoint) else { return (nil, false) }
        var request = URLRequest(url: url)
        request.httpMethod = "GET"
        request.timeoutInterval = timeout

        do {
            let (data, response) = try await URLSession.shared.data(for: request)
            if let httpResponse = response as? HTTPURLResponse, httpResponse.statusCode == 200 {
                return (data, true)
            } else {
                print("⚠️ [Broadlink] Unexpected status for \(url)")
            }
        } catch let error as URLError {
            switch error.code {
            case .timedOut, .cannotFindHost, .cannotConnectToHost, .networkConnectionLost, .notConnectedToInternet:
                print("⚠️ Host not reachable at \(host)")
            case .cancelled:
                break // ignore user-cancelled requests
            default:
                print("⚠️ [Broadlink] \(url) → \(error.localizedDescription)")
            }
        } catch {
            print("⚠️ [Broadlink] \(url) → \(error.localizedDescription)")
        }

        return (nil, false)
    }

    // MARK: - Routes
}
