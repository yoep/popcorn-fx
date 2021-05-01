module popcorn.time.player.adapter {
    requires javafx.graphics;
    requires lombok;
    requires spring.core;

    exports com.github.yoep.player.adapter;
    exports com.github.yoep.player.adapter.listeners;
    exports com.github.yoep.player.adapter.state;
    exports com.github.yoep.player.adapter.subtitles;
}
