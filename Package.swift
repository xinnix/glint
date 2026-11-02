// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "GitCIStatus",
    platforms: [.macOS(.v13)],
    targets: [
        .executableTarget(
            name: "GitCIStatus",
            path: "Sources/GitCIStatus"
        )
    ]
)
