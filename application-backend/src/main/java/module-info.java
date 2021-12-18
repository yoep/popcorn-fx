module popcorn.time.application.backend {
    requires javafx.base;
    requires javafx.graphics;
    requires lombok;
    requires spring.core;
    requires spring.context;
    requires spring.boot.starter.javafx;

    exports com.github.yoep.popcorn.backend.adapters.player;
    exports com.github.yoep.popcorn.backend.adapters.player.state;
    exports com.github.yoep.popcorn.backend.adapters.player.subtitles;
    exports com.github.yoep.popcorn.backend.adapters.player.embaddable;
    exports com.github.yoep.popcorn.backend.adapters.player.listeners;
    exports com.github.yoep.popcorn.backend.adapters.screen;
    exports com.github.yoep.popcorn.backend.adapters.torrent;
    exports com.github.yoep.popcorn.backend.adapters.torrent.model;
    exports com.github.yoep.popcorn.backend.adapters.torrent.state;
    exports com.github.yoep.popcorn.backend.adapters.torrent.listeners;
    exports com.github.yoep.popcorn.backend.adapters.video;
    exports com.github.yoep.popcorn.backend.messages;
}