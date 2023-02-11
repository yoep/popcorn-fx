package com.github.yoep.popcorn.backend;

import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentState;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentStreamState;
import com.github.yoep.popcorn.backend.media.FavoritesSet;
import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.MediaSet;
import com.github.yoep.popcorn.backend.media.StringArray;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteEventCallback;
import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.media.watched.WatchedEventCallback;
import com.github.yoep.popcorn.backend.platform.PlatformInfo;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfoSet;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import com.github.yoep.popcorn.backend.torrent.TorrentStreamEventCallback;
import com.github.yoep.popcorn.backend.torrent.TorrentStreamWrapper;
import com.github.yoep.popcorn.backend.torrent.TorrentWrapper;
import com.github.yoep.popcorn.backend.torrent.TorrentWrapperPointer;
import com.sun.jna.Library;
import com.sun.jna.Native;

/**
 * The Popcorn FX native library interface.
 * Use the {@link FxLib#INSTANCE} to communicate with the loaded library.
 * <p>
 * <i>Example:</i>
 * <pre><code>
 * var subtitles = FxLib.INSTANCE.movie_subtitles(PopcornFxInstance.INSTANCE.get(), movie);
 * </code></pre>
 */
public interface FxLib extends Library {
    FxLib INSTANCE = Native.load("popcorn_fx", FxLib.class);

    PopcornFx new_popcorn_fx();

    PlatformInfo platform_info(PopcornFx instance);

    SubtitleInfoSet default_subtitle_options(PopcornFx instance);

    SubtitleInfoSet movie_subtitles(PopcornFx instance, MovieDetails movie);

    SubtitleInfoSet episode_subtitles(PopcornFx instance, ShowDetails show, Episode episode);

    SubtitleInfoSet filename_subtitles(PopcornFx instance, String filename);

    SubtitleInfo select_or_default_subtitle(PopcornFx instance, SubtitleInfo[] subtitles, int len);

    SubtitleInfo retrieve_preferred_subtitle(PopcornFx instance);

    SubtitleLanguage retrieve_preferred_subtitle_language(PopcornFx instance);

    void update_subtitle(PopcornFx instance, SubtitleInfo subtitle);

    void update_subtitle_custom_file(PopcornFx instance, String filepath);

    void reset_subtitle(PopcornFx instance);

    String download(PopcornFx instance, SubtitleInfo subtitle, SubtitleMatcher matcher);

    Subtitle download_and_parse_subtitle(PopcornFx instance, SubtitleInfo subtitle, SubtitleMatcher matcher);

    Subtitle parse_subtitle(PopcornFx instance, String filePath);

    String subtitle_to_raw(PopcornFx instance, Subtitle subtitle, int type);

    MediaSet retrieve_available_movies(PopcornFx instance, Genre genre, SortBy sort, String keywords, int page);

    MovieDetails retrieve_movie_details(PopcornFx instance, String imdbId);

    void reset_movie_apis(PopcornFx instance);

    MediaSet retrieve_available_shows(PopcornFx instance, Genre genre, SortBy sort, String keywords, int page);

    ShowDetails retrieve_show_details(PopcornFx instance, String imdbId);

    void reset_show_apis(PopcornFx instance);

    FavoritesSet retrieve_available_favorites(PopcornFx instance, Genre genre, SortBy sort, String keywords, int page);

    MediaItem retrieve_favorite_details(PopcornFx instance, String imdbId);

    boolean is_media_liked(PopcornFx instance, MediaItem media);

    FavoritesSet retrieve_all_favorites(PopcornFx instance);

    void add_to_favorites(PopcornFx instance, MediaItem media);

    void remove_from_favorites(PopcornFx instance, MediaItem media);

    void register_favorites_event_callback(PopcornFx instance, FavoriteEventCallback callback);

    String serve_subtitle(PopcornFx instance, Subtitle subtitle, int type);

    void disable_screensaver(PopcornFx instance);

    void enable_screensaver(PopcornFx instance);

    boolean is_media_watched(PopcornFx instance, MediaItem media);

    StringArray retrieve_watched_movies(PopcornFx instance);

    StringArray retrieve_watched_shows(PopcornFx instance);

    void add_to_watched(PopcornFx instance, MediaItem media);

    void remove_from_watched(PopcornFx instance, MediaItem media);

    void register_watched_event_callback(PopcornFx instance, WatchedEventCallback callback);

    TorrentWrapperPointer torrent_wrapper(TorrentWrapper.ByValue torrent);

    void torrent_state_changed(TorrentWrapperPointer torrent, TorrentState state);

    void torrent_piece_finished(TorrentWrapperPointer torrent, int piece);

    TorrentStreamWrapper start_stream(PopcornFx instance, TorrentWrapperPointer torrent);

    void register_torrent_stream_callback(TorrentStreamWrapper stream, TorrentStreamEventCallback callback);

    TorrentStreamState torrent_stream_state(TorrentStreamWrapper stream);

    void dispose_media_item(MediaItem media);

    void dispose_media_items(MediaSet media);

    void dispose_popcorn_fx(PopcornFx instance);
}