package com.github.yoep.popcorn.backend;

import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentState;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentStreamState;
import com.github.yoep.popcorn.backend.controls.PlaybackControlCallback;
import com.github.yoep.popcorn.backend.events.EventC;
import com.github.yoep.popcorn.backend.lib.ByteArray;
import com.github.yoep.popcorn.backend.lib.FxLibInstance;
import com.github.yoep.popcorn.backend.lib.StringArray;
import com.github.yoep.popcorn.backend.logging.LogLevel;
import com.github.yoep.popcorn.backend.media.*;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteEventCallback;
import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.media.watched.WatchedEventCallback;
import com.github.yoep.popcorn.backend.settings.ApplicationConfigEventCallback;
import com.github.yoep.popcorn.backend.settings.models.*;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitleEventCallback;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfoSet;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import com.github.yoep.popcorn.backend.torrent.TorrentStreamEventCallback;
import com.github.yoep.popcorn.backend.torrent.TorrentStreamWrapper;
import com.github.yoep.popcorn.backend.torrent.TorrentWrapper;
import com.github.yoep.popcorn.backend.torrent.TorrentWrapperPointer;
import com.github.yoep.popcorn.backend.torrent.collection.StoredTorrentSet;
import com.github.yoep.popcorn.backend.updater.UpdateCallback;
import com.github.yoep.popcorn.backend.updater.UpdateState;
import com.github.yoep.popcorn.backend.updater.VersionInfo;
import com.sun.jna.Library;
import com.sun.jna.Pointer;

/**
 * The interface for interacting with the Popcorn FX native library.
 * Use the {@link FxLibInstance#INSTANCE} to obtain an instance of this interface
 * for communication with the loaded library.
 * <p>
 * <b>Example Usage:</b>
 * <pre><code>
 * // Obtain an instance of the FxLib interface
 * FxLib fxLib = FxLibInstance.INSTANCE.get();
 *
 * // Access various methods provided by the native library
 * SubtitleInfoSet subtitles = fxLib.movie_subtitles(fxLibInstance, movie);
 * MediaSetResult.ByValue movies = fxLib.retrieve_available_movies(fxLibInstance, Genre.ACTION, SortBy.POPULARITY, "action", 1);
 * // ... and so on
 * </code></pre>
 * <p>
 * This interface defines native methods for various operations related to media, subtitles,
 * torrents, settings, and more. It serves as the bridge between your Java application
 * and the underlying Popcorn FX library.
 */
public interface FxLib extends Library {
    PopcornFx new_popcorn_fx(String[] args, int len);

    SubtitleInfoSet default_subtitle_options(PopcornFx instance);

    SubtitleInfo subtitle_none();

    SubtitleInfo subtitle_custom();

    SubtitleInfoSet movie_subtitles(PopcornFx instance, MovieDetails movie);

    SubtitleInfoSet episode_subtitles(PopcornFx instance, ShowDetails show, Episode episode);

    SubtitleInfoSet filename_subtitles(PopcornFx instance, String filename);

    SubtitleInfo select_or_default_subtitle(PopcornFx instance, SubtitleInfo[] subtitles, int len);

    SubtitleInfo retrieve_preferred_subtitle(PopcornFx instance);

    SubtitleLanguage retrieve_preferred_subtitle_language(PopcornFx instance);

    byte is_subtitle_disabled(PopcornFx instance);

    void update_subtitle(PopcornFx instance, SubtitleInfo subtitle);

    void update_subtitle_custom_file(PopcornFx instance, String filepath);

    void disable_subtitle(PopcornFx instance);

    void reset_subtitle(PopcornFx instance);

    void cleanup_subtitles_directory(PopcornFx instance);

    String download(PopcornFx instance, SubtitleInfo subtitle, SubtitleMatcher matcher);

    Subtitle download_and_parse_subtitle(PopcornFx instance, SubtitleInfo subtitle, SubtitleMatcher matcher);

    void register_subtitle_callback(PopcornFx instance, SubtitleEventCallback callback);

    MediaSetResult.ByValue retrieve_available_movies(PopcornFx instance, Genre genre, SortBy sort, String keywords, int page);

    void reset_movie_apis(PopcornFx instance);

    MediaSetResult.ByValue retrieve_available_shows(PopcornFx instance, Genre genre, SortBy sort, String keywords, int page);

    void reset_show_apis(PopcornFx instance);

    FavoritesSet retrieve_available_favorites(PopcornFx instance, Genre genre, SortBy sort, String keywords, int page);

    MediaResult.ByValue retrieve_media_details(PopcornFx instance, MediaItem media);

    byte is_media_liked(PopcornFx instance, MediaItem media);

    FavoritesSet retrieve_all_favorites(PopcornFx instance);

    void add_to_favorites(PopcornFx instance, MediaItem media);

    void remove_from_favorites(PopcornFx instance, MediaItem media);

    void register_favorites_event_callback(PopcornFx instance, FavoriteEventCallback callback);

    String serve_subtitle(PopcornFx instance, Subtitle subtitle, int type);

    byte is_media_watched(PopcornFx instance, MediaItem media);

    StringArray retrieve_watched_movies(PopcornFx instance);

    StringArray retrieve_watched_shows(PopcornFx instance);

    void add_to_watched(PopcornFx instance, MediaItem media);

    void remove_from_watched(PopcornFx instance, MediaItem media);

    void register_watched_event_callback(PopcornFx instance, WatchedEventCallback callback);

    TorrentWrapperPointer torrent_wrapper(PopcornFx instance, TorrentWrapper.ByValue torrent);

    void torrent_state_changed(TorrentWrapperPointer torrent, TorrentState state);

    void torrent_piece_finished(TorrentWrapperPointer torrent, int piece);

    TorrentStreamWrapper start_stream(PopcornFx instance, TorrentWrapperPointer torrent);

    void stop_stream(PopcornFx instance, TorrentStreamWrapper stream);

    void register_torrent_stream_callback(TorrentStreamWrapper stream, TorrentStreamEventCallback callback);

    TorrentStreamState torrent_stream_state(TorrentStreamWrapper stream);

    Pointer auto_resume_timestamp(PopcornFx instance, String id, String filename);

    void publish_event(PopcornFx instance, EventC.ByValue event);

    void torrent_info(PopcornFx instance, String url);

    byte torrent_collection_is_stored(PopcornFx instance, String magnetUrl);

    StoredTorrentSet torrent_collection_all(PopcornFx instance);

    void torrent_collection_add(PopcornFx instance, String name, String magnetUrl);

    void torrent_collection_remove(PopcornFx instance, String magnetUrl);

    void cleanup_torrents_directory(PopcornFx instance);

    ApplicationSettings application_settings(PopcornFx instance);

    void reload_settings(PopcornFx instance);

    void register_settings_callback(PopcornFx instance, ApplicationConfigEventCallback callback);

    void update_subtitle_settings(PopcornFx instance, SubtitleSettings.ByValue settings);

    void update_torrent_settings(PopcornFx instance, TorrentSettings.ByValue settings);

    void update_ui_settings(PopcornFx instance, UISettings.ByValue settings);

    void update_server_settings(PopcornFx instance, ServerSettings.ByValue settings);

    void update_playback_settings(PopcornFx instance, PlaybackSettings.ByValue settings);

    byte is_youtube_video_player_disabled(PopcornFx instance);

    byte is_fx_video_player_disabled(PopcornFx instance);

    byte is_vlc_video_player_disabled(PopcornFx instance);

    byte is_mouse_disabled(PopcornFx instance);

    byte is_tv_mode(PopcornFx instance);

    byte is_maximized(PopcornFx instance);

    byte is_kiosk_mode(PopcornFx instance);

    VersionInfo version_info(PopcornFx instance);

    UpdateState update_state(PopcornFx instance);

    void check_for_updates(PopcornFx instance);

    void download_update(PopcornFx instance);

    void install_update(PopcornFx instance);

    void register_update_callback(PopcornFx instance, UpdateCallback callback);

    StringArray retrieve_provider_genres(PopcornFx instance, String name);

    StringArray retrieve_provider_sort_by(PopcornFx instance, String name);

    void register_playback_controls(PopcornFx instance, PlaybackControlCallback callback);

    ByteArray poster_placeholder(PopcornFx instance);

    ByteArray artwork_placeholder(PopcornFx instance);

    ByteArray load_fanart(PopcornFx instance, MediaItem item);

    ByteArray load_poster(PopcornFx instance, MediaItem item);

    ByteArray load_image(PopcornFx instance, String url);

    void log(String target, String message, LogLevel level);

    void dispose_media_item(MediaItem media);

    void dispose_media_items(MediaSet media);

    void dispose_torrent_stream(TorrentStreamWrapper wrapper);

    void dispose_subtitle(Subtitle subtitle);

    void dispose_torrent_collection(StoredTorrentSet set);

    void dispose_byte_array(ByteArray byteArray);

    void dispose_popcorn_fx(PopcornFx instance);

    String version();
}