module application {
    requires application.backend;
    requires application.ui;
    requires javafx.graphics;
    requires org.slf4j;
    requires player.popcorn;
    requires uk.co.caprica.vlcj;
    requires video.javafx;
    requires video.vlc;
    requires video.youtube;
    requires static lombok;

    exports com.github.yoep.popcorn;
}