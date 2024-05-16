module video.youtube {
    requires application.backend;
    requires javafx.graphics;
    requires javafx.web;
    requires jdk.jsobject;
    requires org.apache.commons.io;
    requires org.slf4j;

    requires static lombok;

    exports com.github.yoep.video.youtube;
}