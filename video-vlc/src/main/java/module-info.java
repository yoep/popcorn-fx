import uk.co.caprica.vlcj.factory.discovery.provider.DiscoveryDirectoryProvider;

module video.vlc {
    requires application.backend;
    requires com.sun.jna;
    requires javafx.graphics;
    requires org.slf4j;
    requires uk.co.caprica.vlcj.javafx;
    requires uk.co.caprica.vlcj;
    requires vlcj.natives;

    requires static lombok;

    uses DiscoveryDirectoryProvider;

    exports com.github.yoep.video.vlc;
    exports com.github.yoep.video.vlc.discovery;
}