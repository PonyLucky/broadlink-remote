
//  BroadlinkController.swift
//  BroadlinkRemote


import Foundation

struct BLControllerInfo: Decodable {
    let name: String
    let friendly_name: String?
    let ip: String
    let port: Int
    let type: String?
    let mac: String?
    let model: String?
    let devices: [String]?
}

struct BLDeviceInfo: Decodable {
    let name: String
    let friendly_name: String?
    let type: String
    let manufacturer: String?
    let model: String?
}

// Generic node for commands/groups tree
class BLNode: NSObject {
    enum Kind { case group, command }
    let kind: Kind
    let name: String
    let disabled: Bool
    var children: [BLNode] = []
    // For commands we hold the path (dot-separated)
    let commandPath: String?

    init(kind: Kind, name: String, disabled: Bool = false, commandPath: String? = nil, children: [BLNode] = []) {
        self.kind = kind
        self.name = name
        self.disabled = disabled
        self.commandPath = commandPath
        self.children = children
    }
}

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
        URL(string: "http://\(host):\(port)/api\(endpoint)")
    }

    private func makeRequest(_ endpoint: String, method: String, timeout: TimeInterval = 4.0) async -> (Data?, Int?) {
        guard let url = getURL(endpoint) else { return (nil, nil) }
        var request = URLRequest(url: url)
        request.httpMethod = method
        request.timeoutInterval = timeout
        if method == "POST" {
            // No body for command send
            request.setValue("application/json", forHTTPHeaderField: "Accept")
        }
        do {
            let (data, response) = try await URLSession.shared.data(for: request)
            let status = (response as? HTTPURLResponse)?.statusCode
            return (data, status)
        } catch let error as URLError {
            switch error.code {
            case .timedOut, .cannotFindHost, .cannotConnectToHost, .networkConnectionLost, .notConnectedToInternet:
                print("⚠️ Host not reachable at \(host):\(port)")
            case .cancelled:
                break // ignore user-cancelled requests
            default:
                print("⚠️ [Broadlink] \(endpoint) → \(error.localizedDescription)")
            }
        } catch {
            print("⚠️ [Broadlink] \(endpoint) → \(error.localizedDescription)")
        }
        return (nil, nil)
    }

    // MARK: - API
    func fetchControllers() async -> [BLControllerInfo] {
        let (data, status) = await makeRequest("/controller", method: "GET")
        guard status == 200, let data = data else { return [] }
        do {
            return try JSONDecoder().decode([BLControllerInfo].self, from: data)
        } catch {
            print("⚠️ decode controllers: \(error)")
            return []
        }
    }

    func fetchDevices(controller: String) async -> [BLDeviceInfo] {
        let (data, status) = await makeRequest("/\(controller)/device", method: "GET")
        guard status == 200, let data = data else { return [] }
        do {
            return try JSONDecoder().decode([BLDeviceInfo].self, from: data)
        } catch {
            print("⚠️ decode devices: \(error)")
            return []
        }
    }

    // Parse command tree response which is a nested map of groups/commands
    func fetchCommandTree(controller: String, device: String) async -> BLNode? {
        let (data, status) = await makeRequest("/\(controller)/\(device)", method: "GET")
        guard status == 200, let data = data else { return nil }
        do {
            let json = try JSONSerialization.jsonObject(with: data) as? [String: Any]
            // Expect a dict with optional 'groups' and 'commands'
            let root = BLNode(kind: .group, name: device)
            buildNodes(into: root, json: json ?? [:], path: "")
            return root
        } catch {
            print("⚠️ parse command tree: \(error)")
            return nil
        }
    }

    private func buildNodes(into parent: BLNode, json: [String: Any], path: String) {
        // Commands
        if let commands = json["commands"] as? [String: Any] {
            for (name, maybeCmd) in commands.sorted(by: { $0.key < $1.key }) {
                var disabled = false
                if let dict = maybeCmd as? [String: Any], let dis = dict["disabled"] as? Bool { disabled = dis }
                let newPath = path.isEmpty ? name : "\(path).\(name)"
                let node = BLNode(kind: .command, name: name, disabled: disabled, commandPath: newPath)
                parent.children.append(node)
            }
        }
        // Groups
        if let groups = json["groups"] as? [String: Any] {
            for (gname, gval) in groups.sorted(by: { $0.key < $1.key }) {
                let gdict = gval as? [String: Any] ?? [:]
                let disabled = (gdict["disabled"] as? Bool) ?? false
                let node = BLNode(kind: .group, name: gname, disabled: disabled)
                parent.children.append(node)
                buildNodes(into: node, json: gdict, path: path.isEmpty ? gname : "\(path).\(gname)")
            }
        }
    }

    func sendCommand(controller: String, device: String, commandPath: String) async -> Bool {
        let (_, status) = await makeRequest("/\(controller)/\(device)/\(commandPath)", method: "POST")
        return status == 200
    }
}
