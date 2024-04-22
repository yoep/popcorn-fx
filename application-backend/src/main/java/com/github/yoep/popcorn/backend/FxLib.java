package com.github.yoep.popcorn.backend;

import com.github.yoep.popcorn.backend.adapters.screen.FullscreenCallback;
import com.github.yoep.popcorn.backend.adapters.screen.IsFullscreenCallback;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentFileInfoWrapper;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentInfoWrapper;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentState;
import com.github.yoep.popcorn.backend.controls.PlaybackControlCallback;
import com.github.yoep.popcorn.backend.events.EventBridgeCallback;
import com.github.yoep.popcorn.backend.events.EventC;
import com.github.yoep.popcorn.backend.lib.ByteArray;
import com.github.yoep.popcorn.backend.lib.FxLibInstance;
import com.github.yoep.popcorn.backend.lib.StringArray;
import com.github.yoep.popcorn.backend.loader.LoaderEventC;
import com.github.yoep.popcorn.backend.loader.LoaderEventCallback;
import com.github.yoep.popcorn.backend.logging.LogLevel;
import com.github.yoep.popcorn.backend.media.*;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteEventCallback;
import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.media.tracking.AuthorizationOpenCallback;
import com.github.yoep.popcorn.backend.media.tracking.TrackingEventC;
import com.github.yoep.popcorn.backend.media.tracking.TrackingEventCallback;
import com.github.yoep.popcorn.backend.media.watched.WatchedEventCallback;
import com.github.yoep.popcorn.backend.player.*;
import com.github.yoep.popcorn.backend.playlists.Playlist;
import com.github.yoep.popcorn.backend.playlists.PlaylistManagerCallback;
import com.github.yoep.popcorn.backend.playlists.PlaylistManagerEvent;
import com.github.yoep.popcorn.backend.settings.ApplicationConfigEventCallback;
import com.github.yoep.popcorn.backend.settings.models.*;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitleEventCallback;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfoSet;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import com.github.yoep.popcorn.backend.torrent.*;
import com.github.yoep.popcorn.backend.torrent.collection.StoredTorrentSet;
import com.github.yoep.popcorn.backend.updater.UpdateCallback;
import com.github.yoep.popcorn.backend.updater.UpdateState;
import com.github.yoep.popcorn.backend.updater.VersionInfo;
import com.sun.jna.Library;
import com.sun.jna.Pointer;

import java.util.concurrent.atomic.AtomicReference;

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
    AtomicReference<FxLib> INSTANCE = new AtomicReference<>();

    PopcornFx new_popcorn_fx(Pointer args, int len);

    SubtitleInfoSet.ByReference default_subtitle_options(PopcornFx instance);

    SubtitleInfo.ByReference subtitle_none();

    SubtitleInfo.ByReference subtitle_custom();

    SubtitleInfoSet.ByReference movie_subtitles(PopcornFx instance, MovieDetails movie);

    SubtitleInfoSet.ByReference episode_subtitles(PopcornFx instance, ShowDetails show, Episode episode);

    SubtitleInfoSet.ByReference filename_subtitles(PopcornFx instance, String filename);

    SubtitleInfo.ByReference select_or_default_subtitle(PopcornFx instance, SubtitleInfoSet.ByReference subtitleSet);

    SubtitleInfo.ByReference retrieve_preferred_subtitle(PopcornFx instance);

    SubtitleLanguage retrieve_preferred_subtitle_language(PopcornFx instance);

    byte is_subtitle_disabled(PopcornFx instance);

    void update_subtitle(PopcornFx instance, SubtitleInfo subtitle);

    void update_subtitle_custom_file(PopcornFx instance, String filepath);

    void disable_subtitle(PopcornFx instance);

    void reset_subtitle(PopcornFx instance);

    void cleanup_subtitles_directory(PopcornFx instance);

    String download(PopcornFx instance, SubtitleInfo subtitle, SubtitleMatcher.ByValue matcher);

    Subtitle download_and_parse_subtitle(PopcornFx instance, SubtitleInfo subtitle, SubtitleMatcher.ByValue matcher);

    void register_subtitle_callback(PopcornFx instance, SubtitleEventCallback callback);

    MediaSetResult.ByValue retrieve_available_movies(PopcornFx instance, Genre genre, SortBy sort, String keywords, int page);

    void reset_movie_apis(PopcornFx instance);

    MediaSetResult.ByValue retrieve_available_shows(PopcornFx instance, Genre genre, SortBy sort, String keywords, int page);

    void reset_show_apis(PopcornFx instance);

    FavoritesSet retrieve_available_favorites(PopcornFx instance, Genre genre, SortBy sort, String keywords, int page);

    MediaResult.ByValue retrieve_media_details(PopcornFx instance, MediaItem media);

    byte is_media_liked(PopcornFx instance, MediaItem.ByReference media);

    void add_to_favorites(PopcornFx instance, MediaItem media);

    void remove_from_favorites(PopcornFx instance, MediaItem media);

    void register_favorites_event_callback(PopcornFx instance, FavoriteEventCallback callback);

    String serve_subtitle(PopcornFx instance, SubtitleInfo.ByReference subtitleInfo, SubtitleMatcher.ByValue matcher, int type);

    byte is_media_watched(PopcornFx instance, MediaItem media);

    StringArray.ByReference retrieve_watched_movies(PopcornFx instance);

    StringArray.ByReference retrieve_watched_shows(PopcornFx instance);

    void add_to_watched(PopcornFx instance, MediaItem media);

    void remove_from_watched(PopcornFx instance, MediaItem media);

    void register_watched_event_callback(PopcornFx instance, WatchedEventCallback callback);

    void torrent_resolve_info_callback(PopcornFx instance, ResolveTorrentInfoCallback callback);

    void register_torrent_resolve_callback(PopcornFx instance, ResolveTorrentCallback callback);

    void torrent_cancel_callback(PopcornFx instance, CancelTorrentCallback callback);

    Long register_torrent_stream_event_callback(PopcornFx instance, Long streamHandle, TorrentStreamEventCallback callback);

    void remove_torrent_stream_event_callback(PopcornFx instance, Long streamHandle, Long callbackHandle);

    void torrent_state_changed(PopcornFx instance, String handle, TorrentState state);

    void torrent_piece_finished(PopcornFx instance, String handle, int piece);

    void torrent_download_status(PopcornFx instance, String handle, DownloadStatusC.ByValue downloadStatus);

    void publish_event(PopcornFx instance, EventC.ByValue event);

    void register_event_callback(PopcornFx instance, EventBridgeCallback callback);

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

    byte is_fx_video_player_disabled(PopcornFx instance);

    byte is_mouse_disabled(PopcornFx instance);

    byte is_tv_mode(PopcornFx instance);

    byte is_maximized(PopcornFx instance);

    byte is_kiosk_mode(PopcornFx instance);
    
    byte is_youtube_video_player_enabled(PopcornFx instance);
    
    byte is_vlc_video_player_enabled(PopcornFx instance);

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

    Long play_playlist(PopcornFx instance, Playlist.ByValue set);

    void register_playlist_manager_callback(PopcornFx instance, PlaylistManagerCallback callback);

    Long play_next_playlist_item(PopcornFx instance);

    void stop_playlist(PopcornFx instance);

    Playlist.ByValue playlist(PopcornFx instance);

    PlayerWrapper active_player(PopcornFx instance);

    void set_active_player(PopcornFx instance, String playerId);

    PlayerSet players(PopcornFx instance);

    PlayerWrapper player_by_id(PopcornFx instance, String playerId);

    PlayerWrapperPointer player_pointer_by_id(PopcornFx instance, String playerId);

    void register_player_callback(PopcornFx instance, PlayerManagerCallback callback);

    void register_player(PopcornFx instance, PlayerWrapperRegistration.ByValue player);

    void invoke_player_event(PlayerWrapperPointer wrapper, PlayerEventC.ByValue event);

    void remove_player(PopcornFx instance, String playerId);

    void player_pause(PlayerWrapperPointer ptr);

    void player_resume(PlayerWrapperPointer ptr);

    void player_seek(PlayerWrapperPointer ptr, long time);

    void player_stop(PlayerWrapperPointer ptr);

    void register_loader_callback(PopcornFx instance, LoaderEventCallback callback);

    Long loader_load(PopcornFx instance, String url);

    Long loader_load_torrent_file(PopcornFx instance, TorrentInfoWrapper.ByValue torrentInfo, TorrentFileInfoWrapper.ByValue torrentFile);

    void loader_cancel(PopcornFx instance, Long handle);

    void register_is_fullscreen_callback(PopcornFx instance, IsFullscreenCallback callback);

    void register_fullscreen_callback(PopcornFx instance, FullscreenCallback callback);
    
    void register_tracking_authorization_open(PopcornFx instance, AuthorizationOpenCallback callback);

    void register_tracking_provider_callback(PopcornFx instance, TrackingEventCallback callback);
    
    byte tracking_is_authorized(PopcornFx instance);
    
    void tracking_authorize(PopcornFx instance);
    
    void tracking_disconnect(PopcornFx instance);
    
    void discover_external_players(PopcornFx instance);

    void log(String target, String message, LogLevel level);

    void dispose_subtitle_info_set(SubtitleInfoSet.ByReference set);

    void dispose_subtitle_info(SubtitleInfo.ByReference info);

    void dispose_media_item(MediaItem.ByReference media);

    void dispose_media_item_value(MediaItem.ByValue media);

    void dispose_media_items(MediaSet.ByValue media);

    void dispose_subtitle(Subtitle.ByReference subtitle);

    void dispose_torrent_collection(StoredTorrentSet set);

    void dispose_byte_array(ByteArray byteArray);

    void dispose_string_array(StringArray array);

    void dispose_event_value(EventC.ByValue event);

    void dispose_favorites(FavoritesSet favorites);

    void dispose_player_manager_event(PlayerManagerEvent.ByValue event);

    void dispose_player_pointer(PlayerWrapperPointer ptr);

    void dispose_player_event_value(PlayerEventC.ByValue event);

    void dispose_player(PlayerWrapper.ByReference wrapper);

    void dispose_loader_event_value(LoaderEventC.ByValue event);

    void dispose_playlist_manager_event_value(PlaylistManagerEvent.ByValue event);

    void dispose_torrent_stream_event_value(TorrentStreamEventC.ByValue event);

    void dispose_tracking_event_value(TrackingEventC.ByValue event);

    void dispose_popcorn_fx(PopcornFx instance);

    String version();
}