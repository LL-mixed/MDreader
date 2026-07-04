import SwiftUI

private struct RepositoryKey: EnvironmentKey {
    static let defaultValue: DocRepository? = nil
}

private struct ZoomStoreKey: EnvironmentKey {
    static let defaultValue: ZoomStore? = nil
}

private struct ThemeStoreKey: EnvironmentKey {
    static let defaultValue: ThemeStore? = nil
}

private struct SessionStoreKey: EnvironmentKey {
    static let defaultValue: SessionStore? = nil
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
    var mdThemeStore: ThemeStore? {
        get { self[ThemeStoreKey.self] }
        set { self[ThemeStoreKey.self] = newValue }
    }
    var mdSessionStore: SessionStore? {
        get { self[SessionStoreKey.self] }
        set { self[SessionStoreKey.self] = newValue }
    }
}
