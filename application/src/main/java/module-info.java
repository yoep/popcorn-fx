module application {
    requires application.backend;
    requires application.ui;
    requires com.sun.jna;
    requires javafx.graphics;
    requires org.apache.httpcomponents.httpclient;
    requires org.slf4j;
    requires player.popcorn;
    requires torrent.frostwire;
    requires uk.co.caprica.vlcj;
    requires video.javafx;
    requires video.vlc;
    requires video.youtube;

    exports com.github.yoep.popcorn;
}