import SwiftUI

private struct RepositoryKey: EnvironmentKey {
    static let defaultValue: DocRepository? = nil
}

private struct ZoomStoreKey: EnvironmentKey {
    static let defaultValue: ZoomStore? = nil
}

extension EnvironmentValues {
    var mdRepository: DocRepository? {
        get { self[RepositoryKey.self] }
        set { self[RepositoryKey.self] = newValue }
    }
    var mdZoomStore: ZoomStore? {
        get { self[ZoomStoreKey.self] }
        set { self[ZoomStoreKey.self] = newValue }
    }
}
