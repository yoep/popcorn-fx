module application.backend {
    requires ch.qos.logback.classic;
    requires ch.qos.logback.core;
    requires com.sun.jna;
    requires java.annotation;
    requires java.desktop;
    requires javafx.base;
    requires javafx.graphics;
    requires org.slf4j;

    requires static lombok;

    exports com.github.yoep.popcorn.backend.adapters.platform;
    exports com.github.yoep.popcorn.backend.adapters.player.embaddable;
    exports com.github.yoep.popcorn.backend.adapters.player.listeners;
    exports com.github.yoep.popcorn.backend.adapters.player.state;
    exports com.github.yoep.popcorn.backend.adapters.player;
    exports com.github.yoep.popcorn.backend.adapters.screen;
    exports com.github.yoep.popcorn.backend.adapters.torrent.model;
    exports com.github.yoep.popcorn.backend.adapters.torrent.state;
    exports com.github.yoep.popcorn.backend.adapters.torrent;
    exports com.github.yoep.popcorn.backend.adapters.video.listeners;
    exports com.github.yoep.popcorn.backend.adapters.video.state;
    exports com.github.yoep.popcorn.backend.adapters.video;
    exports com.github.yoep.popcorn.backend.controls;
    exports com.github.yoep.popcorn.backend.events;
    exports com.github.yoep.popcorn.backend.info;
    exports com.github.yoep.popcorn.backend.lib;
    exports com.github.yoep.popcorn.backend.loader;
    exports com.github.yoep.popcorn.backend.logging;
    exports com.github.yoep.popcorn.backend.media.favorites;
    exports com.github.yoep.popcorn.backend.media.filters.model;
    exports com.github.yoep.popcorn.backend.media.providers;
    exports com.github.yoep.popcorn.backend.media.tracking;
    exports com.github.yoep.popcorn.backend.media.watched;
    exports com.github.yoep.popcorn.backend.media;
    exports com.github.yoep.popcorn.backend.messages;
    exports com.github.yoep.popcorn.backend.player;
    exports com.github.yoep.popcorn.backend.playlists.model;
    exports com.github.yoep.popcorn.backend.playlists;
    exports com.github.yoep.popcorn.backend.services;
    exports com.github.yoep.popcorn.backend.settings.models.subtitles;
    exports com.github.yoep.popcorn.backend.settings.models;
    exports com.github.yoep.popcorn.backend.settings;
    exports com.github.yoep.popcorn.backend.subtitles.listeners;
    exports com.github.yoep.popcorn.backend.subtitles.model;
    exports com.github.yoep.popcorn.backend.subtitles;
    exports com.github.yoep.popcorn.backend.torrent.collection;
    exports com.github.yoep.popcorn.backend.torrent;
    exports com.github.yoep.popcorn.backend.updater;
    exports com.github.yoep.popcorn.backend.utils;
    exports com.github.yoep.popcorn.backend;

    exports com.github.yoep.popcorn.backend.playlists.ffi to com.sun.jna;

    opens com.github.yoep.popcorn.backend.player to com.sun.jna;
    opens com.github.yoep.popcorn.backend.playlists to com.sun.jna;
    opens com.github.yoep.popcorn.backend.subtitles to com.sun.jna;
    opens com.github.yoep.popcorn.backend.torrent to com.sun.jna;
    opens com.github.yoep.popcorn.backend.updater to com.sun.jna;
    opens com.github.yoep.popcorn.backend.playlists.model to com.sun.jna;
    opens com.github.yoep.popcorn.backend.subtitles.ffi to com.sun.jna;
    exports com.github.yoep.popcorn.backend.subtitles.ffi;
}