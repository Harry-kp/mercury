// Launch-time and memory benchmark for macOS apps.
//
//   swift scripts/bench-launch.swift /Applications/Mercury.app [/Applications/Postman.app ...]
//
// For each app, RUNS times: start its executable, time until its first
// on-screen window, wait IDLE seconds, then sum the resident memory of the
// whole process tree (Electron apps run several helper processes; counting
// only the main one flatters them), and quit it. Also reports the bundle's
// size on disk. Nothing is clicked or typed. Run 1 is cold (after install or
// reboot); later runs are warm. Results are the median of the warm runs.
import AppKit

let RUNS = 4
let IDLE: UInt32 = 5

func shell(_ args: [String]) -> String {
    let p = Process()
    p.executableURL = URL(fileURLWithPath: args[0])
    p.arguments = Array(args.dropFirst())
    let pipe = Pipe()
    p.standardOutput = pipe
    p.standardError = FileHandle.nullDevice
    try? p.run()
    p.waitUntilExit()
    return String(data: pipe.fileHandleForReading.readDataToEndOfFile(), encoding: .utf8) ?? ""
}

/// Every pid in the tree rooted at `root`.
func tree(_ root: Int32) -> [Int32] {
    var children: [Int32: [Int32]] = [:]
    for line in shell(["/bin/ps", "-A", "-o", "pid=,ppid="]).split(separator: "\n") {
        let f = line.split(separator: " ").compactMap { Int32($0) }
        if f.count == 2 { children[f[1], default: []].append(f[0]) }
    }
    var out = [root], i = 0
    while i < out.count { out += children[out[i]] ?? []; i += 1 }
    return out
}

func rssKB(_ pids: [Int32]) -> Int {
    let list = pids.map(String.init).joined(separator: ",")
    return shell(["/bin/ps", "-o", "rss=", "-p", list])
        .split(separator: "\n").compactMap { Int($0.trimmingCharacters(in: .whitespaces)) }.reduce(0, +)
}

func firstWindowMs(_ pids: Set<Int32>, since t0: Date) -> Double? {
    while Date().timeIntervalSince(t0) < 30 {
        let list = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as! [[String: Any]]
        let hit = list.contains { w in
            guard let pid = w[kCGWindowOwnerPID as String] as? Int32, (w[kCGWindowLayer as String] as? Int) == 0,
                  let b = w[kCGWindowBounds as String] as? [String: Any],
                  ((b["Width"] as? NSNumber)?.doubleValue ?? 0) > 200 else { return false }
            return pids.contains(pid)
        }
        if hit { return Date().timeIntervalSince(t0) * 1000 }
        usleep(2000)
    }
    return nil
}

func median(_ xs: [Double]) -> Double {
    let s = xs.sorted(); return s.isEmpty ? .nan : s[s.count / 2]
}

print("| App | Launch to window (warm, median) | First launch | Memory, idle (all processes) | Size on disk |")
print("|---|---|---|---|---|")
for path in CommandLine.arguments.dropFirst() {
    guard let bundle = Bundle(path: path), let exe = bundle.executableURL else {
        print("| \(path) | not an app bundle | | | |"); continue
    }
    let name = (bundle.infoDictionary?["CFBundleName"] as? String) ?? exe.lastPathComponent
    var launch: [Double] = [], mem: [Double] = []
    var first = Double.nan
    for run in 0..<RUNS {
        let p = Process()
        p.executableURL = exe
        var env = ProcessInfo.processInfo.environment
        env["MERCURY_HOME"] = NSTemporaryDirectory() + "mercury-bench"  // isolates Mercury's own data
        p.environment = env
        let t0 = Date()
        guard (try? p.run()) != nil else { break }
        let ms = firstWindowMs(Set(tree(p.processIdentifier)).union([p.processIdentifier]), since: t0)
        // helpers may appear after the window; collect the tree again after idling
        sleep(IDLE)
        let kb = rssKB(tree(p.processIdentifier))
        for pid in tree(p.processIdentifier).reversed() { kill(pid, SIGTERM) }
        p.waitUntilExit()
        sleep(2)
        guard let ms else { continue }
        if run == 0 { first = ms } else { launch.append(ms); mem.append(Double(kb)) }
    }
    let sizeKB = Int(shell(["/usr/bin/du", "-sk", path]).split(separator: "\t").first ?? "0") ?? 0
    print(String(format: "| %@ | %.0f ms | %.0f ms | %.0f MB | %.0f MB |",
                 name, median(launch), first, median(mem) / 1024, Double(sizeKB) / 1024))
}
