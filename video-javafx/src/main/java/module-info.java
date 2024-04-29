module video.javafx {
    requires application.backend;
    requires javafx.base;
    requires javafx.graphics;
    requires javafx.media;
    requires org.slf4j;

    requires static lombok;

    exports com.github.yoep.video.javafx;
}