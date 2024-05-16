module torrent.frostwire {
    requires application.backend;
    requires java.annotation;
    requires java.desktop;
    requires javafx.base;
    requires jlibtorrent;
    requires org.apache.commons.io;
    requires org.apache.httpcomponents.httpclient;
    requires org.apache.httpcomponents.httpcore;
    requires org.slf4j;

    requires static lombok;

    exports com.github.yoep.torrent.frostwire;
}