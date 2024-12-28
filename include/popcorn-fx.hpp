#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>


/// The available categories of [crate::core::media::Media] items.
/// These can be used as filter to retrieve data from the API.
enum class Category : int32_t {
  Movies = 0,
  Series = 1,
  Favorites = 2,
};

/// The cleaning mode for downloaded files.
enum class CleaningMode : int32_t {
  /// Cleaning is disabled.
  Off = 0,
  /// Files are cleaned on application shutdown.
  OnShutdown = 1,
  /// Files are cleaned when fully watched.
  Watched = 2,
};

/// The decoration to apply to the subtitle during rendering.
enum class DecorationType : int32_t {
  None = 0,
  Outline = 1,
  OpaqueBackground = 2,
  SeeThroughBackground = 3,
};

enum class LoadingState : uint32_t {
  Initializing,
  Starting,
  RetrievingSubtitles,
  DownloadingSubtitle,
  RetrievingMetadata,
  VerifyingFiles,
  Connecting,
  Downloading,
  DownloadFinished,
  Ready,
  Playing,
  Cancelled,
};

/// The C-compatible logging level for log messages sent over FFI.
///
/// This enum represents the different logging levels that can be used to send log messages from Rust to C code.
/// It includes five different levels of logging: `Trace`, `Debug`, `Info`, `Warn`, and `Error`.
enum class LogLevel : int32_t {
  Off = 0,
  Trace = 1,
  Debug = 2,
  Info = 3,
  Warn = 4,
  Error = 5,
};

/// The C compatible media error types.
enum class MediaErrorC : int32_t {
  Failed = 0,
  NoItemsFound = 1,
  NoAvailableProviders = 2,
};

enum class MediaTrackingSyncState : int32_t {
  Success = 0,
  Failed = 1,
};

/// Events related to playback control, triggered by the media system of the OS.
/// These events can be used to modify the player state based on the given media event.
enum class PlaybackControlEvent : int32_t {
  TogglePlaybackState = 0,
  Forward = 1,
  Rewind = 2,
};

/// The playback state of the current media item.
///
/// This enum describes the different states of the playback known by the player.
enum class PlaybackState : int32_t {
  /// This is the initial state and indicates that the playback state is unknown or hasn't been received from the player.
  ///
  /// This state usually occurs when the player is starting up or there is no active media item.
  UNKNOWN = -1,
  /// The media player is ready to start playback.
  READY = 0,
  /// The media player is currently loading the media item.
  LOADING = 1,
  /// The media player is currently buffering the media data.
  BUFFERING = 2,
  /// The media player is currently playing the media item.
  PLAYING = 3,
  /// The media player has paused the playback.
  PAUSED = 4,
  /// The media player has stopped the playback.
  STOPPED = 5,
  /// An error has occurred during playback.
  ERROR = 6,
};

/// An enumeration representing the possible states of a player.
enum class PlayerState : int32_t {
  Unknown = -1,
  Ready = 0,
  Loading = 1,
  Buffering = 2,
  Playing = 3,
  Paused = 4,
  Stopped = 5,
  Error = 6,
};

/// An enumeration representing the state of a playlist.
///
/// The `PlaylistState` enum is used to indicate the current state of a playlist, such as whether it's idle, playing, stopped, completed, or in an error state.
enum class PlaylistState : int32_t {
  Idle,
  Playing,
  Stopped,
  Completed,
  Error,
};

/// The playback quality defined in a resolution size
enum class Quality {
  P480,
  P720,
  P1080,
  P2160,
};

/// The supported subtitle fonts to use for rendering subtitles.
enum class SubtitleFamily : int32_t {
  Arial = 0,
  ComicSans = 1,
  Georgia = 2,
  Tahoma = 3,
  TrebuchetMs = 4,
  Verdana = 5,
};

/// The supported subtitle languages.
enum class SubtitleLanguage : int32_t {
  None = 0,
  Custom = 1,
  Arabic = 2,
  Bulgarian = 3,
  Bosnian = 4,
  Czech = 5,
  Danish = 6,
  German = 7,
  ModernGreek = 8,
  English = 9,
  Spanish = 10,
  Estonian = 11,
  Basque = 12,
  Persian = 13,
  Finnish = 14,
  French = 15,
  Hebrew = 16,
  Croatian = 17,
  Hungarian = 18,
  Indonesian = 19,
  Italian = 20,
  Lithuanian = 21,
  Dutch = 22,
  Norwegian = 23,
  Polish = 24,
  Portuguese = 25,
  PortugueseBrazil = 26,
  Romanian = 27,
  Russian = 28,
  Slovene = 29,
  Serbian = 30,
  Swedish = 31,
  Thai = 32,
  Turkish = 33,
  Ukrainian = 34,
  Vietnamese = 35,
};

/// Represents the different states of torrent health.
enum class TorrentHealthState : uint32_t {
  /// Unknown health state, indicating that the health of the torrent could not be determined.
  Unknown,
  /// Bad health state, indicating that the torrent is in poor condition.
  Bad,
  /// Medium health state, indicating that the torrent is in a moderate condition.
  Medium,
  /// Good health state, indicating that the torrent is in good condition.
  Good,
  /// Excellent health state, indicating that the torrent is in excellent condition.
  Excellent,
};

/// The state of the [TorrentStream].
enum class TorrentStreamState : int32_t {
  /// The initial state of the torrent stream.
  /// This state indicates that the stream is preparing the initial pieces.
  Preparing = 0,
  /// The torrent can be streamed over HTTP.
  Streaming = 1,
  /// The torrent has been stopped and can not longer be streamed.
  Stopped = 2,
};

/// The C compatible update state
enum class UpdateStateC : int32_t {
  CheckingForNewVersion = 0,
  UpdateAvailable = 1,
  NoUpdateAvailable = 2,
  Downloading = 3,
  /// Indicates that the download has finished.
  DownloadFinished = 4,
  Installing = 5,
  InstallationFinished = 6,
  Error = 7,
};

template<typename T = void>
struct Box;

/// A type representing a callback function to set the fullscreen state of the application.
struct FullscreenCallback;

struct PlayerWrapperC;

/// The [PopcornFX] application instance.
/// This is the main entry into the FX application and manages all known data.
///
/// # Examples
///
/// Create a simple instance with default values.
/// This instance will have the [log4rs] loggers initialized.
/// ```no_run
/// use popcorn_fx::PopcornFX;
/// let instance = PopcornFX::default();
/// ```
struct PopcornFX;

/// A C-compatible struct representing a player.
struct PlayerC {
  /// A pointer to a null-terminated C string representing the player's unique identifier (ID).
  char *id;
  /// A pointer to a null-terminated C string representing the name of the player.
  char *name;
  /// A pointer to a null-terminated C string representing the description of the player.
  char *description;
  /// A pointer to the graphic resource data.
  uint8_t *graphic_resource;
  /// The length of the graphic resource data.
  int32_t graphic_resource_len;
  /// The state of the player.
  PlayerState state;
  /// Indicates whether embedded playback is supported by the player.
  bool embedded_playback_supported;
};

struct RatingC {
  uint16_t percentage;
  uint32_t watching;
  uint32_t votes;
  uint32_t loved;
  uint32_t hated;
};

/// The C compatible [Images] representation.
struct ImagesC {
  char *poster;
  char *fanart;
  char *banner;
};

struct MovieOverviewC {
  char *title;
  char *imdb_id;
  char *year;
  RatingC *rating;
  ImagesC images;
};

/// A C-compatible struct representing torrent information.
struct TorrentMediaInfoC {
  /// A pointer to a null-terminated C string representing the torrent URL.
  char *url;
  /// A pointer to a null-terminated C string representing the torrent provider.
  char *provider;
  /// A pointer to a null-terminated C string representing the torrent source.
  char *source;
  /// A pointer to a null-terminated C string representing the torrent title.
  char *title;
  /// A pointer to a null-terminated C string representing the torrent quality.
  char *quality;
  /// The number of seeders for the torrent.
  uint32_t seed;
  /// The number of peers for the torrent.
  uint32_t peer;
  /// A pointer to a null-terminated C string representing the torrent size in bytes.
  char *size;
  /// A pointer to a null-terminated C string representing the torrent filesize in human-readable format.
  char *filesize;
  /// A pointer to a null-terminated C string representing the selected file within the torrent collection.
  char *file;
};

struct TorrentQualityC {
  char *quality;
  TorrentMediaInfoC torrent;
};

struct TorrentEntryC {
  char *language;
  TorrentQualityC *qualities;
  int32_t len;
};

/// The C compatible [MovieDetails] representation
///
/// Use the [MovieDetails::from] to convert the C instance back to a rust struct.
struct MovieDetailsC {
  char *title;
  char *imdb_id;
  char *year;
  RatingC *rating;
  ImagesC images;
  char *synopsis;
  int32_t runtime;
  char *trailer;
  char **genres;
  int32_t genres_len;
  TorrentEntryC *torrents;
  int32_t torrents_len;
};

struct ShowOverviewC {
  char *imdb_id;
  char *tvdb_id;
  char *title;
  char *year;
  int32_t num_seasons;
  ImagesC images;
  RatingC *rating;
};

/// The C compatible [Episode] media information.
struct EpisodeC {
  int32_t season;
  int32_t episode;
  int64_t first_aired;
  char *title;
  char *synopsis;
  char *tvdb_id;
  char *thumb;
  TorrentQualityC *torrents;
  int32_t len;
};

struct ShowDetailsC {
  char *imdb_id;
  char *tvdb_id;
  char *title;
  char *year;
  int32_t num_seasons;
  ImagesC images;
  RatingC *rating;
  char *synopsis;
  int32_t runtime;
  char *status;
  char **genres;
  int32_t genres_len;
  EpisodeC *episodes;
  int32_t episodes_len;
};

/// A C-compatible holder for a media item, which may represent a movie, show, or episode.
struct MediaItemC {
  /// A pointer to the movie overview struct.
  MovieOverviewC *movie_overview;
  /// A pointer to the movie details struct.
  MovieDetailsC *movie_details;
  /// A pointer to the show overview struct.
  ShowOverviewC *show_overview;
  /// A pointer to the show details struct.
  ShowDetailsC *show_details;
  /// A pointer to the episode struct.
  EpisodeC *episode;
};

/// The C compatible subtitle settings.
struct SubtitleSettingsC {
  /// The directory path for storing subtitles
  char *directory;
  /// Indicates if the subtitle directory will be cleaned
  /// when the application is closed
  bool auto_cleaning;
  /// The default selected subtitle language
  SubtitleLanguage default_subtitle;
  /// The subtitle font to use
  SubtitleFamily font_family;
  /// The subtitle font size to use
  uint32_t font_size;
  /// The subtitle rendering decoration type
  DecorationType decoration;
  /// Indicates if the subtitle should be rendered in a bold font
  bool bold;
};

/// The C compatible torrent settings.
struct TorrentSettingsC {
  /// The torrent directory to store the torrents
  char *directory;
  /// Indicates if the torrents directory will be cleaned on closure
  CleaningMode cleaning_mode;
  /// The max number of connections
  uint32_t connections_limit;
  /// The download rate limit
  uint32_t download_rate_limit;
  /// The upload rate limit
  uint32_t upload_rate_limit;
};

/// The UI scale of the application
struct UiScale {
  float value;
};

/// The C compatible ui settings
struct UiSettingsC {
  /// The default language of the application
  char *default_language;
  /// The ui scale of the application
  UiScale ui_scale;
  /// The default start screen of the application
  Category start_screen;
  /// The indication if the UI was maximized the last time the application was closed
  bool maximized;
  /// The indication if the UI should use a native window rather than the borderless stage
  bool native_window_enabled;
};

/// The C compatible server settings.
struct ServerSettingsC {
  /// The configured api server to use, can be `ptr::null()`
  char *api_server;
};

/// The C compatible playback settings
struct PlaybackSettingsC {
  /// The default playback quality
  Quality *quality;
  /// Indicates if the playback will be opened in fullscreen mode
  bool fullscreen;
  /// Indicates if the next episode of the show will be played
  bool auto_play_next_episode_enabled;
};

/// Represents the C-compatible struct for the last sync.
struct LastSyncC {
  /// The number of non-leap seconds since January 1, 1970 0:00:00 UTC = Unix timestamp.
  int64_t time;
  /// The state of media tracking sync.
  MediaTrackingSyncState state;
};

/// Represents the C-compatible struct for tracking settings.
struct TrackingSettingsC {
  /// Pointer to the last sync.
  LastSyncC *last_sync;
};

/// The C compatible application settings.
struct PopcornSettingsC {
  /// The subtitle settings of the application
  SubtitleSettingsC subtitle_settings;
  /// The torrent settings of the application
  TorrentSettingsC torrent_settings;
  /// The ui settings of the application
  UiSettingsC ui_settings;
  /// The api server settings of the application
  ServerSettingsC server_settings;
  /// The playback settings of the application
  PlaybackSettingsC playback_settings;
  /// The tracking settings of the application
  TrackingSettingsC tracking_settings;
};

/// A C-compatible byte array that can be used to return byte array data from Rust functions.
///
/// This struct contains a pointer to the byte array data and the length of the byte array.
/// It is intended for use in C code that needs to interact with Rust functions that return byte array data.
struct ByteArray {
  /// A pointer to the byte array data.
  uint8_t *values;
  /// The length of the byte array.
  int32_t len;
};

/// Represents the health statistics of a torrent.
struct TorrentHealth {
  /// The health state of the torrent.
  TorrentHealthState state;
  /// The ratio of uploaded data to downloaded data for the torrent.
  float ratio;
  /// The number of seeders (peers with a complete copy of the torrent).
  uint32_t seeds;
  /// The number of leechers currently downloading the torrent.
  uint32_t leechers;
};

/// The C compatible [SubtitleFile] representation.
struct SubtitleFileC {
  int32_t file_id;
  char *name;
  char *url;
  float score;
  int32_t downloads;
  const int32_t *quality;
};

/// The C compatible [SubtitleInfo] representation.
struct SubtitleInfoC {
  /// The IMDB ID if known, this can be [ptr::null]
  char *imdb_id;
  SubtitleLanguage language;
  SubtitleFileC *files;
  int32_t len;
};

/// The C array of available [SubtitleInfo].
struct SubtitleInfoSet {
  /// The available subtitle array
  SubtitleInfoC *subtitles;
  /// The length of the array
  int32_t len;
};

/// A C-compatible struct representing a player change event.
struct PlayerChangedEventC {
  /// The (nullable) old player id
  char *old_player_id;
  /// The new player id
  char *new_player_id;
  /// The new player name
  char *new_player_name;
};

/// A C-compatible struct representing torrent file information.
struct TorrentFileInfoC {
  /// A pointer to a null-terminated C string representing the filename.
  char *filename;
  /// A pointer to a null-terminated C string representing the file path.
  char *file_path;
  /// The size of the file in bytes.
  uint64_t file_size;
  /// The index of the file.
  uint32_t file_index;
};

/// A C-compatible set/array of items.
///
/// This struct is used to represent a set of items that can be passed between Rust and C code.
/// It includes a pointer to the items and their length.
template<typename T>
struct CArray {
  /// A pointer to the array of items.
  T *items;
  /// The length of the array.
  int32_t len;
};

/// A C-compatible struct representing torrent information.
struct TorrentInfoC {
  /// The underlying torrent handle
  int64_t handle;
  char *info_hash;
  /// A pointer to a null-terminated C string representing the URI of the torrent.
  char *uri;
  /// A pointer to a null-terminated C string representing the name of the torrent.
  char *name;
  /// A pointer to a null-terminated C string representing the directory name of the torrent.
  char *directory_name;
  /// The total number of files in the torrent.
  uint32_t total_files;
  /// A set of `TorrentFileInfoC` structs representing individual files within the torrent.
  CArray<TorrentFileInfoC> files;
};

/// The C compatible [Event] representation.
struct EventC {
  enum class Tag {
    /// Invoked when the player is changed
    /// 1st argument is the new player id, 2nd argument is the new player name
    PlayerChanged,
    /// Invoked when the player playback has started for a new media item
    PlayerStarted,
    /// Invoked when the player is being stopped
    PlayerStopped,
    /// Invoked when the playback state is changed
    PlaybackStateChanged,
    /// Invoked when the watch state of an item is changed
    /// 1st argument is a pointer to the imdb id (C string), 2nd argument is a boolean indicating the new watch state
    WatchStateChanged,
    /// Invoked when the loading of a media item has started
    LoadingStarted,
    /// Invoked when the loading of a media item has completed
    LoadingCompleted,
    /// Invoked when the torrent details have been loaded
    TorrentDetailsLoaded,
    /// Invoked when the player should be closed
    ClosePlayer,
  };

  struct PlayerChanged_Body {
    PlayerChangedEventC _0;
  };

  struct PlaybackStateChanged_Body {
    PlaybackState _0;
  };

  struct WatchStateChanged_Body {
    char *_0;
    bool _1;
  };

  struct TorrentDetailsLoaded_Body {
    TorrentInfoC _0;
  };

  Tag tag;
  union {
    PlayerChanged_Body player_changed;
    PlaybackStateChanged_Body playback_state_changed;
    WatchStateChanged_Body watch_state_changed;
    TorrentDetailsLoaded_Body torrent_details_loaded;
  };
};

struct VecFavoritesC {
  MovieOverviewC *movies;
  int32_t movies_len;
  ShowOverviewC *shows;
  int32_t shows_len;
};

/// A C-compatible struct representing the event when loading starts.
/// A C-compatible struct representing the event when loading starts.
struct LoadingStartedEventC {
  /// The URL of the media being loaded.
  char *url;
  /// The title or name of the media being loaded.
  char *title;
  /// The URL of a thumbnail image associated with the media, or `ptr::null()` if not available.
  char *thumbnail;
  /// The URL of a background image associated with the media, or `ptr::null()` if not available.
  char *background;
  /// The quality or resolution information of the media, or `ptr::null()` if not available.
  char *quality;
};

struct LoadingProgressC {
  /// Progress indication between 0 and 1 that represents the progress of the download.
  float progress;
  /// The number of seeds available for the torrent.
  uint32_t seeds;
  /// The number of peers connected to the torrent.
  uint32_t peers;
  /// The total download transfer rate in bytes of payload only, not counting protocol chatter.
  uint32_t download_speed;
  /// The total upload transfer rate in bytes of payload only, not counting protocol chatter.
  uint32_t upload_speed;
  /// The total amount of data downloaded in bytes.
  uint64_t downloaded;
  /// The total size of the torrent in bytes.
  uint64_t total_size;
};

/// A C-compatible enum representing loading errors.
struct LoadingErrorC {
  enum class Tag {
    /// Error indicating a parsing failure with an associated error message.
    ParseError,
    /// Error indicating a torrent-related failure with an associated error message.
    TorrentError,
    /// Error indicating a media-related failure with an associated error message.
    MediaError,
    /// Error indicating a timeout with an associated error message.
    TimeoutError,
    InvalidData,
    Cancelled,
  };

  struct ParseError_Body {
    char *_0;
  };

  struct TorrentError_Body {
    char *_0;
  };

  struct MediaError_Body {
    char *_0;
  };

  struct TimeoutError_Body {
    char *_0;
  };

  struct InvalidData_Body {
    char *_0;
  };

  Tag tag;
  union {
    ParseError_Body parse_error;
    TorrentError_Body torrent_error;
    MediaError_Body media_error;
    TimeoutError_Body timeout_error;
    InvalidData_Body invalid_data;
  };
};

/// A C-compatible enum representing loader events.
struct LoaderEventC {
  enum class Tag {
    LoadingStarted,
    StateChanged,
    ProgressChanged,
    LoaderError,
  };

  struct LoadingStarted_Body {
    int64_t _0;
    LoadingStartedEventC _1;
  };

  struct StateChanged_Body {
    int64_t _0;
    LoadingState _1;
  };

  struct ProgressChanged_Body {
    int64_t _0;
    LoadingProgressC _1;
  };

  struct LoaderError_Body {
    int64_t _0;
    LoadingErrorC _1;
  };

  Tag tag;
  union {
    LoadingStarted_Body loading_started;
    StateChanged_Body state_changed;
    ProgressChanged_Body progress_changed;
    LoaderError_Body loader_error;
  };
};

/// Structure defining a set of media items.
/// Each media items is separated in a specific implementation array.
struct MediaSetC {
  /// The movie media items array.
  MovieOverviewC *movies;
  int32_t movies_len;
  /// The show media items array.
  ShowOverviewC *shows;
  int32_t shows_len;
};

/// A C-compatible enum representing player events.
struct PlayerEventC {
  enum class Tag {
    DurationChanged,
    TimeChanged,
    StateChanged,
    VolumeChanged,
  };

  struct DurationChanged_Body {
    uint64_t _0;
  };

  struct TimeChanged_Body {
    uint64_t _0;
  };

  struct StateChanged_Body {
    PlayerState _0;
  };

  struct VolumeChanged_Body {
    uint32_t _0;
  };

  Tag tag;
  union {
    DurationChanged_Body duration_changed;
    TimeChanged_Body time_changed;
    StateChanged_Body state_changed;
    VolumeChanged_Body volume_changed;
  };
};

struct PlaySubtitleRequestC {
  bool enabled;
  SubtitleInfoC *info;
};

/// Represents a play request in C-compatible form.
struct PlayRequestC {
  /// The URL of the media to be played.
  char *url;
  /// The title of the media.
  char *title;
  /// The optional caption of the media, or [ptr::null_mut] if not available.
  char *caption;
  /// The optional URL of the thumbnail image for the media, or [ptr::null_mut] if not available.
  char *thumb;
  /// The URL of the background image for the media, or [ptr::null_mut] if not available.
  char *background;
  /// The quality of the media, or [ptr::null_mut] if not available.
  char *quality;
  /// Pointer to a mutable u64 value representing the auto-resume timestamp.
  uint64_t *auto_resume_timestamp;
  /// The stream handle pointer of the play request.
  /// This handle can be used to retrieve more information about the underlying stream.
  int64_t *stream_handle;
  /// The subtitle playback information for this request
  PlaySubtitleRequestC subtitle;
};

/// Represents events related to player management in C-compatible form.
struct PlayerManagerEventC {
  enum class Tag {
    /// Indicates a change in the active player.
    ActivePlayerChanged,
    /// Indicates a change in the players set.
    PlayersChanged,
    /// Indicates that the active player's playback has been changed
    PlayerPlaybackChanged,
    /// Indicates a change in the duration of a player.
    PlayerDurationChanged,
    /// Indicates a change in the playback time of a player.
    PlayerTimeChanged,
    /// Indicates a change in the state of a player.
    PlayerStateChanged,
  };

  struct ActivePlayerChanged_Body {
    PlayerChangedEventC _0;
  };

  struct PlayerPlaybackChanged_Body {
    PlayRequestC _0;
  };

  struct PlayerDurationChanged_Body {
    uint64_t _0;
  };

  struct PlayerTimeChanged_Body {
    uint64_t _0;
  };

  struct PlayerStateChanged_Body {
    PlayerState _0;
  };

  Tag tag;
  union {
    ActivePlayerChanged_Body active_player_changed;
    PlayerPlaybackChanged_Body player_playback_changed;
    PlayerDurationChanged_Body player_duration_changed;
    PlayerTimeChanged_Body player_time_changed;
    PlayerStateChanged_Body player_state_changed;
  };
};

/// A C-compatible struct representing a playlist item.
struct PlaylistItemC {
  /// The URL of the playlist item.
  char *url;
  /// The title of the playlist item.
  char *title;
  /// The caption/subtitle of the playlist item.
  char *caption;
  /// The thumbnail URL of the playlist item.
  char *thumb;
  /// The quality information of the playlist item.
  char *quality;
  /// A pointer to the parent media item, if applicable.
  MediaItemC *parent_media;
  /// A pointer to the media item associated with the playlist item.
  MediaItemC *media;
  /// A pointer to the timestamp for auto-resume, if applicable.
  const uint64_t *auto_resume_timestamp;
  /// A boolean flag indicating whether subtitles are enabled for the playlist item.
  bool subtitles_enabled;
  /// A pointer to the subtitle information for the playlist item, if available, else [ptr::null_mut()].
  SubtitleInfoC *subtitle_info;
  /// A pointer to the torrent information for the playlist item, if applicable, else [ptr::null_mut()].
  TorrentInfoC *torrent_info;
  /// A pointer to the torrent file information for the playlist item, if applicable, else [ptr::null_mut()].
  TorrentFileInfoC *torrent_file_info;
};

/// A C-compatible struct representing information about the next item to be played.
struct PlayingNextInfoC {
  /// A pointer to the timestamp indicating when the next item will be played.
  const uint64_t *playing_in;
  /// A pointer to the next playlist item.
  PlaylistItemC *next_item;
};

/// A C-compatible enum representing different playlist manager events.
struct PlaylistManagerEventC {
  enum class Tag {
    /// Represents a playlist change event.
    PlaylistChanged,
    /// Represents an event indicating the next item to be played.
    PlayingNext,
    /// Represents a state change event in the playlist manager.
    StateChanged,
  };

  struct PlayingNext_Body {
    PlayingNextInfoC _0;
  };

  struct StateChanged_Body {
    PlaylistState _0;
  };

  Tag tag;
  union {
    PlayingNext_Body playing_next;
    StateChanged_Body state_changed;
  };
};

/// The C compatible string array.
/// It's mainly used for returning string arrays as result of C function calls.
struct StringArray {
  /// The string array
  char **values;
  /// The length of the string array
  int32_t len;
};

struct StyledTextC {
  char *text;
  bool italic;
  bool bold;
  bool underline;
};

struct SubtitleLineC {
  StyledTextC *texts;
  int32_t len;
};

/// Represents a cue in a subtitle track in a C-compatible format.
struct SubtitleCueC {
  /// A pointer to a null-terminated C string representing the cue identifier.
  char *id;
  /// The start time of the cue in milliseconds.
  uint64_t start_time;
  /// The end time of the cue in milliseconds.
  uint64_t end_time;
  /// A pointer to an array of subtitle lines.
  SubtitleLineC *lines;
  /// The number of lines in the cue.
  int32_t number_of_lines;
};

/// The parsed subtitle representation for C.
/// It contains the data of a subtitle file that can be displayed.
struct SubtitleC {
  /// The filepath that has been parsed
  char *file;
  /// The info of the parsed subtitle if available, else [ptr::null_mut]
  SubtitleInfoC *info;
  /// The parsed cues from the subtitle file
  SubtitleCueC *cues;
  /// The total number of cue elements
  int32_t len;
};

/// Represents user preferences for subtitles.
struct SubtitlePreference {
  enum class Tag {
    /// Specifies a preferred subtitle language.
    Language,
    /// Indicates subtitles are disabled.
    Disabled,
  };

  struct Language_Body {
    SubtitleLanguage _0;
  };

  Tag tag;
  union {
    Language_Body language;
  };
};

/// The C compatible struct for [MagnetInfo].
struct MagnetInfoC {
  /// The name of the magnet
  char *name;
  /// The magnet uri to the torrent
  char *magnet_uri;
};

/// The collection of stored magnets.
/// It contains the C compatible information for [std::ffi].
struct TorrentCollectionSet {
  /// The array of magnets
  MagnetInfoC *magnets;
  /// The length of the array
  int32_t len;
};

struct DownloadStatusC {
  /// Progress indication between 0 and 1 that represents the progress of the download.
  float progress;
  /// The number of seeds available for the torrent.
  uint32_t seeds;
  /// The number of peers connected to the torrent.
  uint32_t peers;
  /// The total download transfer rate in bytes of payload only, not counting protocol chatter.
  uint32_t download_speed;
  /// The total upload transfer rate in bytes of payload only, not counting protocol chatter.
  uint32_t upload_speed;
  /// The total amount of data downloaded in bytes.
  uint64_t downloaded;
  /// The total size of the torrent in bytes.
  uint64_t total_size;
};

/// Represents a torrent stream event in C-compatible form.
struct TorrentStreamEventC {
  enum class Tag {
    /// Indicates a change in the state of the torrent stream.
    StateChanged,
    /// Indicates a change in the download status of the torrent stream.
    DownloadStatus,
  };

  struct StateChanged_Body {
    TorrentStreamState _0;
  };

  struct DownloadStatus_Body {
    DownloadStatusC _0;
  };

  Tag tag;
  union {
    StateChanged_Body state_changed;
    DownloadStatus_Body download_status;
  };
};

/// Represents a C-compatible tracking event.
struct TrackingEventC {
  enum class Tag {
    /// Authorization state change event.
    AuthorizationStateChanged,
  };

  struct AuthorizationStateChanged_Body {
    bool _0;
  };

  Tag tag;
  union {
    AuthorizationStateChanged_Body authorization_state_changed;
  };
};

/// The subtitle matcher C compatible struct.
/// It contains the information which should be matched when selecting a subtitle file to load.
struct SubtitleMatcherC {
  /// The nullable name of the media item.
  char *name;
  /// The nullable quality of the media item.
  /// This can be represented as `720p` or `720`.
  char *quality;
};

/// A C-compatible handle representing a loading process.
///
/// This type is used to represent a loading process and is exposed as a C-compatible handle.
/// It points to the memory location where loading process information is stored in a C context.
using LoadingHandleC = const int64_t*;

/// Represents a set of players in C-compatible form.
struct PlayerSet {
  /// Pointer to an array of player instances.
  PlayerC *players;
  /// Length of the player array.
  int32_t len;
};

/// A type alias for a C-compatible callback function that takes an `EventC` parameter.
///
/// This type alias is used to define functions in Rust that can accept C callback functions
/// with the specified signature.
using EventCCallback = void(*)(EventC);

struct FavoriteEventC {
  enum class Tag {
    /// Event indicating that the like state of a media item changed.
    ///
    /// * `*mut c_char`   - The imdb id of the media item that changed.
    /// * `bool`            - The new like state of the media item.
    LikedStateChanged,
  };

  struct LikedStateChanged_Body {
    char *_0;
    bool _1;
  };

  Tag tag;
  union {
    LikedStateChanged_Body liked_state_changed;
  };
};

/// Type definition for a callback function that checks if the application is in fullscreen mode.
///
/// The `IsFullscreenCallback` type represents an external callback function that returns a boolean value to indicate
/// whether the application is in fullscreen mode.
using IsFullscreenCallback = bool(*)();

/// A C-compatible callback function type for loader events.
using LoaderEventCallback = void(*)(LoaderEventC);

/// The C compatible callback for playback control events.
using PlaybackControlsCallbackC = void(*)(PlaybackControlEvent);

/// A C-compatible callback function type for player play events.
using PlayerPlayCallback = void(*)(PlayRequestC);

/// A C-compatible callback function type for player pause events.
using PlayerPauseCallback = void(*)();

/// A C-compatible callback function type for player resume events.
using PlayerResumeCallback = void(*)();

/// A C-compatible callback function type for player seek events.
using PlayerSeekCallback = void(*)(uint64_t);

/// A C-compatible callback function type for player stop events.
using PlayerStopCallback = void(*)();

/// A C-compatible struct representing player registration information.
struct PlayerRegistrationC {
  /// A pointer to a null-terminated C string representing the player's unique identifier (ID).
  char *id;
  /// A pointer to a null-terminated C string representing the name of the player.
  char *name;
  /// A pointer to a null-terminated C string representing the description of the player.
  char *description;
  /// A Pointer to the graphic resource of the player.
  /// Use graphic_resource_len to get the length of the graphic resource byte array.
  uint8_t *graphic_resource;
  /// The length of the graphic resource array.
  int32_t graphic_resource_len;
  /// The state of the player.
  PlayerState state;
  /// Indicates whether embedded playback is supported by the player.
  bool embedded_playback_supported;
  /// A callback function pointer for the "play" action.
  PlayerPlayCallback play_callback;
  /// A callback function pointer for the "pause" action.
  PlayerPauseCallback pause_callback;
  /// A callback function pointer for the "resume" action.
  PlayerResumeCallback resume_callback;
  /// A callback function pointer for the "seek" action.
  PlayerSeekCallback seek_callback;
  /// A callback function pointer for the "stop" action.
  PlayerStopCallback stop_callback;
};

/// A C-compatible callback function type for player manager events.
using PlayerManagerEventCallback = void(*)(PlayerManagerEventC);

/// The callback function type for playlist manager events in C.
///
/// This type represents a C-compatible function pointer that can be used to handle playlist manager events.
/// When invoked, it receives a `PlaylistManagerEventC` as its argument.
using PlaylistManagerCallbackC = void(*)(PlaylistManagerEventC);

/// The C compatible application events.
struct ApplicationConfigEventC {
  enum class Tag {
    /// Invoked when the application settings have been reloaded or loaded
    SettingsLoaded,
    /// Invoked when the subtitle settings have been changed
    SubtitleSettingsChanged,
    /// Invoked when the torrent settings have been changed
    TorrentSettingsChanged,
    /// Invoked when the ui settings have been changed
    UiSettingsChanged,
    /// Invoked when the server settings have been changed
    ServerSettingsChanged,
    /// Invoked when the playback settings have been changed
    PlaybackSettingsChanged,
    /// Invoked when the tracking settings have been changed
    TrackingSettingsChanged,
  };

  struct SubtitleSettingsChanged_Body {
    SubtitleSettingsC _0;
  };

  struct TorrentSettingsChanged_Body {
    TorrentSettingsC _0;
  };

  struct UiSettingsChanged_Body {
    UiSettingsC _0;
  };

  struct ServerSettingsChanged_Body {
    ServerSettingsC _0;
  };

  struct PlaybackSettingsChanged_Body {
    PlaybackSettingsC _0;
  };

  struct TrackingSettingsChanged_Body {
    TrackingSettingsC _0;
  };

  Tag tag;
  union {
    SubtitleSettingsChanged_Body subtitle_settings_changed;
    TorrentSettingsChanged_Body torrent_settings_changed;
    UiSettingsChanged_Body ui_settings_changed;
    ServerSettingsChanged_Body server_settings_changed;
    PlaybackSettingsChanged_Body playback_settings_changed;
    TrackingSettingsChanged_Body tracking_settings_changed;
  };
};

/// The C callback for the setting events.
using ApplicationConfigCallbackC = void(*)(ApplicationConfigEventC);

/// The C compatible [SubtitleEvent] representation
struct SubtitleEventC {
  enum class Tag {
    PreferenceChanged,
  };

  struct PreferenceChanged_Body {
    SubtitlePreference _0;
  };

  Tag tag;
  union {
    PreferenceChanged_Body preference_changed;
  };
};

/// The C callback for the subtitle events.
using SubtitleCallbackC = void(*)(SubtitleEventC);

/// Type alias for the C-compatible authorization open function.
using AuthorizationOpenC = bool(*)(char *uri);

/// Type alias for the C-compatible tracking event callback function.
using TrackingEventCCallback = void(*)(TrackingEventC event);

/// The C compatible representation of the application runtime information.
struct PatchInfoC {
  /// The runtime version of the application.
  char *version;
};

/// The C compatible representation of version information from the update channel.
struct VersionInfoC {
  /// The latest release version on the update channel.
  PatchInfoC application;
  /// The runtime version of the application.
  PatchInfoC runtime;
};

/// The C-compatible representation of the [DownloadProgress] struct.
///
/// This struct is used to provide C code access to the download progress of an update event.
///
/// # Fields
///
/// * `total_size` - The total size of the update download in bytes.
/// * `downloaded` - The total number of bytes downloaded so far.
struct DownloadProgressC {
  uint64_t total_size;
  uint64_t downloaded;
};

/// The C-compatible representation of the [InstallationProgress] struct.
///
/// This struct is used to provide C code access to the installation progress of an update event.
///
/// # Fields
///
/// * `task` - The current task being executed during the installation process.
/// * `total_tasks` - The total number of tasks that need to be executed during the installation process.
/// * `task_progress` - The current progress of the current task, represented as a fraction between 0.0 and 1.0.
struct InstallationProgressC {
  uint16_t task;
  uint16_t total_tasks;
};

/// The C compatible representation of the update events.
///
/// This enum maps to the `UpdateEvent` enum but with C-compatible data types.
///
/// # Fields
///
/// * `StateChanged(state)` - Invoked when the state of the updater has changed
/// * `UpdateAvailable(version)` - Invoked when a new update is available
/// * `DownloadProgress(progress)` - Invoked when the update download progresses
struct UpdateEventC {
  enum class Tag {
    StateChanged,
    UpdateAvailable,
    DownloadProgress,
    InstallationProgress,
  };

  struct StateChanged_Body {
    UpdateStateC _0;
  };

  struct UpdateAvailable_Body {
    VersionInfoC _0;
  };

  struct DownloadProgress_Body {
    DownloadProgressC _0;
  };

  struct InstallationProgress_Body {
    InstallationProgressC _0;
  };

  Tag tag;
  union {
    StateChanged_Body state_changed;
    UpdateAvailable_Body update_available;
    DownloadProgress_Body download_progress;
    InstallationProgress_Body installation_progress;
  };
};

/// The C compatible callback for update events.
using UpdateCallbackC = void(*)(UpdateEventC);

struct WatchedEventC {
  enum class Tag {
    /// Event indicating that the watched state of a media item changed.
    ///
    /// * `*mut c_char`   - The imdb id of the media item that changed.
    /// * `bool`            - The new watched state of the media item.
    WatchedStateChanged,
  };

  struct WatchedStateChanged_Body {
    char *_0;
    bool _1;
  };

  Tag tag;
  union {
    WatchedStateChanged_Body watched_state_changed;
  };
};

struct GenreC {
  char *key;
  char *text;
};

struct SortByC {
  char *key;
  char *text;
};

/// The C compatible media result for an array of media items.
struct MediaSetResult {
  enum class Tag {
    Ok,
    Err,
  };

  struct Ok_Body {
    MediaSetC _0;
  };

  struct Err_Body {
    MediaErrorC _0;
  };

  Tag tag;
  union {
    Ok_Body ok;
    Err_Body err;
  };
};

/// The C-compatible media result for a single media item.
struct MediaResult {
  enum class Tag {
    Ok,
    Err,
  };

  struct Ok_Body {
    MediaItemC _0;
  };

  struct Err_Body {
    MediaErrorC _0;
  };

  Tag tag;
  union {
    Ok_Body ok;
    Err_Body err;
  };
};

/// A C-compatible enum representing various errors related to torrents.
struct TorrentErrorC {
  enum class Tag {
    /// Represents an error indicating an invalid URL.
    InvalidUrl,
    /// Represents an error indicating a file not found.
    FileNotFound,
    /// Represents a generic file-related error.
    FileError,
    /// Represents an error indicating an invalid stream state.
    InvalidStreamState,
    /// Represents an error indicating an invalid handle.
    InvalidHandle,
    /// Represents an error indicating failure during torrent resolving.
    TorrentResolvingFailed,
    /// Represents an error indicating failure during torrent collection loading.
    TorrentCollectionLoadingFailed,
    /// Represent a general torrent error failure
    Torrent,
  };

  struct InvalidUrl_Body {
    char *_0;
  };

  struct FileNotFound_Body {
    char *_0;
  };

  struct FileError_Body {
    char *_0;
  };

  struct InvalidStreamState_Body {
    TorrentStreamState _0;
  };

  struct InvalidHandle_Body {
    char *_0;
  };

  struct TorrentResolvingFailed_Body {
    char *_0;
  };

  struct TorrentCollectionLoadingFailed_Body {
    char *_0;
  };

  struct Torrent_Body {
    char *_0;
  };

  Tag tag;
  union {
    InvalidUrl_Body invalid_url;
    FileNotFound_Body file_not_found;
    FileError_Body file_error;
    InvalidStreamState_Body invalid_stream_state;
    InvalidHandle_Body invalid_handle;
    TorrentResolvingFailed_Body torrent_resolving_failed;
    TorrentCollectionLoadingFailed_Body torrent_collection_loading_failed;
    Torrent_Body torrent;
  };
};

/// A C-compatible enum representing either a successful result or an error.
template<typename T, typename E>
struct ResultC {
  enum class Tag {
    /// Represents a successful result containing a value of type `T`.
    Ok,
    /// Represents an error containing a value of type `E`.
    Err,
  };

  struct Ok_Body {
    T _0;
  };

  struct Err_Body {
    E _0;
  };

  Tag tag;
  union {
    Ok_Body ok;
    Err_Body err;
  };
};


extern "C" {

/// Retrieve a pointer to the active player as a `PlayerC` instance from the PopcornFX player manager.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `popcorn_fx` pointer.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
///
/// # Returns
///
/// Returns a pointer to a `PlayerC` instance representing the active player, or a null pointer if there is no active player.
PlayerC *active_player(PopcornFX *popcorn_fx);

/// Add the media item to the favorites.
/// Duplicate favorite media items are ignored.
void add_to_favorites(PopcornFX *popcorn_fx, const MediaItemC *favorite);

/// Add the given media item to the watched list.
void add_to_watched(PopcornFX *popcorn_fx, const MediaItemC *watchable);

/// Retrieve the application settings.
/// These are the setting preferences of the users for the popcorn FX instance.
PopcornSettingsC *application_settings(PopcornFX *popcorn_fx);

/// Retrieve the default artwork (placeholder) image data as a C-compatible byte array.
///
/// This function returns a C-compatible byte array containing the data for the default artwork (placeholder) image.
/// The default artwork image is typically used as a fallback when an artwork image is not available for a media item or is still being loaded.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to a `PopcornFX` instance.
///
/// # Safety
///
/// This function should only be called from C code, and the returned byte array should be disposed of using the `dispose_byte_array` function.
ByteArray *artwork_placeholder(PopcornFX *popcorn_fx);

/// Calculates the health of a torrent based on the number of seeds and leechers.
///
/// # Returns
///
/// Returns the health of the torrent.
TorrentHealth *calculate_torrent_health(PopcornFX *popcorn_fx, uint32_t seeds, uint32_t leechers);

/// Start polling the update channel for new application versions.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to a `PopcornFX` instance.
void check_for_updates(PopcornFX *popcorn_fx);

/// Clean the subtitles directory.
///
/// # Safety
///
/// This function should only be called from C code.
/// The `popcorn_fx` pointer must be valid and properly initialized.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
void cleanup_subtitles_directory(PopcornFX *popcorn_fx);

/// Clean the torrents directory.
/// This will remove all existing torrents from the system.
void cleanup_torrents_directory(PopcornFX *popcorn_fx);

/// Retrieve the default options available for the subtitles.
///
/// # Safety
///
/// This function should only be called from C code.
/// The `popcorn_fx` pointer must be valid and properly initialized.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
///
/// # Returns
///
/// A pointer to a `SubtitleInfoSet` instance.
SubtitleInfoSet *default_subtitle_options(PopcornFX *popcorn_fx);

/// Starts the discovery process for external players such as VLC and DLNA servers.
void discover_external_players(PopcornFX *popcorn_fx);

/// Frees the memory allocated for the given C-compatible byte array.
///
/// This function should be called from C code in order to free memory that has been allocated by Rust.
///
/// # Safety
///
/// This function should only be called on C-compatible byte arrays that have been allocated by Rust.
void dispose_byte_array(Box<ByteArray> array);

/// Dispose of the given event from the event bridge.
///
/// This function takes ownership of a boxed `EventC` object, releasing its resources.
///
/// # Arguments
///
/// * `event` - A boxed `EventC` object to be disposed of.
void dispose_event_value(EventC event);

/// Dispose of a C-compatible favorites collection.
///
/// This function is responsible for cleaning up resources associated with a C-compatible favorites collection.
///
/// # Arguments
///
/// * `favorites` - A C-compatible favorites collection to be disposed of.
void dispose_favorites(Box<VecFavoritesC> favorites);

/// Dispose of a C-compatible LoaderEventC value.
///
/// This function is responsible for cleaning up resources associated with a C-compatible LoaderEventC value.
///
/// # Arguments
///
/// * `event` - A C-compatible LoaderEventC value to be disposed of.
void dispose_loader_event_value(LoaderEventC event);

/// Dispose of a C-compatible MediaItemC value wrapped in a Box.
///
/// This function is responsible for cleaning up resources associated with a C-compatible MediaItemC value
/// wrapped in a Box.
///
/// # Arguments
///
/// * `media` - A Box containing a C-compatible MediaItemC value to be disposed of.
void dispose_media_item(Box<MediaItemC> media);

/// Dispose of a C-compatible MediaItemC value.
///
/// This function is responsible for cleaning up resources associated with a C-compatible MediaItemC value.
///
/// # Arguments
///
/// * `media` - A C-compatible MediaItemC value to be disposed of.
void dispose_media_item_value(MediaItemC media);

/// Dispose of a C-compatible media set.
///
/// This function is responsible for cleaning up resources associated with a C-compatible media set.
///
/// # Arguments
///
/// * `media` - A C-compatible media set to be disposed of.
void dispose_media_items(MediaSetC media);

/// Disposes of the `PlayerC` instance and deallocates its memory.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++),
/// and the caller is responsible for ensuring the safety of the provided `player` pointer.
///
/// # Arguments
///
/// * `player` - A box containing the `PlayerC` instance to be disposed of.
void dispose_player(Box<PlayerC> player);

/// Disposes of the `PlayerEventC` instance and deallocates its memory.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++),
/// and the caller is responsible for ensuring the safety of the provided `event` pointer.
///
/// # Arguments
///
/// * `event` - A box containing the `PlayerEventC` instance to be disposed of.
void dispose_player_event_value(PlayerEventC event);

/// Dispose of a C-compatible player manager event.
///
/// This function is responsible for cleaning up resources associated with a C-compatible player manager event.
///
/// # Arguments
///
/// * `event` - A C-compatible player manager event to be disposed of.
void dispose_player_manager_event(PlayerManagerEventC event);

/// Disposes of the `PlayerWrapperC` instance and deallocates its memory.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++),
/// and the caller is responsible for ensuring the safety of the provided `ptr` pointer.
///
/// # Arguments
///
/// * `ptr` - A box containing the `PlayerWrapperC` instance to be disposed of.
void dispose_player_pointer(Box<PlayerWrapperC> ptr);

/// Dispose of a playlist item.
///
/// # Arguments
///
/// * `item` - A boxed `PlaylistItemC` representing the item to be disposed of.
void dispose_playlist_item(Box<PlaylistItemC> item);

/// Dispose of a C-compatible PlaylistManagerEventC value.
///
/// This function is responsible for cleaning up resources associated with a C-compatible PlaylistManagerEventC value.
///
/// # Arguments
///
/// * `event` - A C-compatible PlaylistManagerEventC value to be disposed of.
void dispose_playlist_manager_event_value(PlaylistManagerEventC event);

/// Dispose of a C-style array of playlist items.
///
/// This function takes ownership of a C-style array of `PlaylistItemC` and drops it to free the associated memory.
///
/// # Arguments
///
/// * `set` - A boxed C-style array of `PlaylistItemC` to be disposed of.
void dispose_playlist_set(Box<CArray<PlaylistItemC>> set);

/// Delete the PopcornFX instance, given as a [ptr], in a safe way.
/// All data within the instance will be deleted from memory making the instance unusable.
/// This means that the original pointer will become invalid.
void dispose_popcorn_fx(Box<PopcornFX> instance);

/// Dispose of a C-compatible string array.
///
/// This function takes ownership of a boxed `StringArray` object, releasing its resources.
///
/// # Arguments
///
/// * `array` - A boxed `StringArray` object to be disposed of.
void dispose_string_array(Box<StringArray> array);

/// Frees the memory allocated for the `SubtitleC` structure.
///
/// # Safety
///
/// This function is marked as `unsafe` because it's assumed that the `SubtitleC` structure was allocated using `Box`,
/// and dropping a `Box` pointing to valid memory is safe. However, if the `SubtitleC` was allocated in a different way
/// or if the memory was already deallocated, calling this function could lead to undefined behavior.
void dispose_subtitle(Box<SubtitleC> subtitle);

/// Frees the memory allocated for the `SubtitleInfoC` structure.
///
/// # Safety
///
/// This function is marked as `unsafe` because it's assumed that the `SubtitleInfoC` structure was allocated using `Box`,
/// and dropping a `Box` pointing to valid memory is safe. However, if the `SubtitleInfoC` was allocated in a different way
/// or if the memory was already deallocated, calling this function could lead to undefined behavior.
void dispose_subtitle_info(Box<SubtitleInfoC> info);

/// Frees the memory allocated for the `SubtitleInfoSet` structure.
///
/// # Safety
///
/// This function is marked as `unsafe` because it's assumed that the `SubtitleInfoSet` structure was allocated using `Box`,
/// and dropping a `Box` pointing to valid memory is safe. However, if the `SubtitleInfoSet` was allocated in a different way
/// or if the memory was already deallocated, calling this function could lead to undefined behavior.
void dispose_subtitle_info_set(Box<SubtitleInfoSet> set);

/// Frees the memory allocated for the `SubtitlePreference` structure.
///
/// # Safety
///
/// This function is marked as `unsafe` because it's assumed that the `SubtitlePreference` structure was allocated using `Box`,
/// and dropping a `Box` pointing to valid memory is safe. However, if the `SubtitlePreference` was allocated in a different way
/// or if the memory was already deallocated, calling this function could lead to undefined behavior.
///
void dispose_subtitle_preference(Box<SubtitlePreference> subtitle_preference);

/// Dispose the [TorrentCollectionSet] from memory.
void dispose_torrent_collection(Box<TorrentCollectionSet> collection_set);

void dispose_torrent_health(Box<TorrentHealth> health);

void dispose_torrent_stream_event_value(TorrentStreamEventC event);

/// Disposes a tracking event value.
///
/// # Arguments
///
/// * `event` - The tracking event to be disposed.
void dispose_tracking_event_value(TrackingEventC event);

/// Download the given [SubtitleInfo] based on the best match according to the [SubtitleMatcher].
///
/// It returns the filepath to the subtitle on success, else [ptr::null_mut].
char *download(PopcornFX *popcorn_fx, const SubtitleInfoC *subtitle, const SubtitleMatcherC *matcher);

/// Download and parse the given subtitle info.
///
/// It returns the [SubtitleC] reference on success, else [ptr::null_mut].
SubtitleC *download_and_parse_subtitle(PopcornFX *popcorn_fx, const SubtitleInfoC *subtitle, const SubtitleMatcherC *matcher);

/// Start downloading the application update if available.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to a `PopcornFX` instance.
void download_update(PopcornFX *popcorn_fx);

/// Retrieve the given subtitles for the given episode.
///
/// This function takes a reference to the `PopcornFX` instance, a reference to a `ShowDetailsC`, and a reference
/// to an `EpisodeC` for which subtitles are to be retrieved.
/// It returns a reference to `SubtitleInfoSet` containing the available subtitles for the episode.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the `PopcornFX` instance.
/// * `show` - A reference to the `ShowDetailsC` containing information about the show.
/// * `episode` - A reference to the `EpisodeC` for which subtitles are to be retrieved.
///
/// # Returns
///
/// A pointer to the `SubtitleInfoSet` containing the available subtitles for the episode.
/// <i>The returned reference should be managed by the caller.</i>
SubtitleInfoSet *episode_subtitles(PopcornFX *popcorn_fx, const ShowDetailsC *show, const EpisodeC *episode);

/// Retrieve the available subtitles for the given filename
SubtitleInfoSet *filename_subtitles(PopcornFX *popcorn_fx, char *filename);

/// Install the latest available update.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to a `PopcornFX` instance.
void install_update(PopcornFX *popcorn_fx);

/// Invoke a player event on a wrapped player instance.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `player` pointer.
///
/// # Arguments
///
/// * `player` - A mutable reference to a `PlayerWrapperC` instance that wraps a player.
/// * `event` - The player event to invoke.
///
/// # Notes
///
/// This function checks if the `player` instance exists and is of the expected type (`PlayerWrapper`).
/// If the conditions are met, it invokes the specified player event on the wrapped player.
void invoke_player_event(PlayerWrapperC *player, PlayerEventC event);

/// Verify if the FX embedded video player has been disabled.
bool is_fx_video_player_enabled(PopcornFX *popcorn_fx);

/// Verify if the application should started in kiosk mode.
/// The behavior of kiosk mode is dependant on the UI implementation and not delegated by the backend.
bool is_kiosk_mode(PopcornFX *popcorn_fx);

/// Verify if the application should be maximized on startup.
bool is_maximized(PopcornFX *popcorn_fx);

/// Verify if the given media item is liked/favorite of the user.
/// It will use the first non [ptr::null_mut] field from the [MediaItemC] struct.
///
/// It will return false if all fields in the [MediaItemC] are [ptr::null_mut].
bool is_media_liked(PopcornFX *popcorn_fx, MediaItemC *favorite);

/// Verify if the given media item is watched by the user.
///
/// It returns true when the item is watched, else false.
bool is_media_watched(PopcornFX *popcorn_fx, const MediaItemC *watchable);

/// Verify if the application mouse should be disabled.
/// The disabling of the mouse should be implemented by the UI implementation and has no behavior on
/// the backend itself.
bool is_mouse_disabled(PopcornFX *popcorn_fx);

/// Verify if the TV mode is activated for the application.
bool is_tv_mode(PopcornFX *popcorn_fx);

/// Checks if the VLC video player is enabled in the PopcornFX options.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
///
/// # Returns
///
/// `true` if the VLC video player is enabled, otherwise `false`.
bool is_vlc_video_player_enabled(PopcornFX *popcorn_fx);

/// Checks if the YouTube video player is enabled in the PopcornFX options.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
///
/// # Returns
///
/// `true` if the YouTube video player is enabled, otherwise `false`.
bool is_youtube_video_player_enabled(PopcornFX *popcorn_fx);

/// Loads the fanart image data for the given media item.
///
/// This function should be called from C code in order to load fanart image data for a media item.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to the `PopcornFX` instance that will load the image data.
/// * `media` - a C-compatible media item holder that contains information about the media item to load.
///
/// # Returns
///
/// If fanart image data is available for the media item, a C-compatible byte array containing the image data is returned.
/// Otherwise, a placeholder byte array is returned.
///
/// # Safety
///
/// This function should only be called from C code, and the returned byte array should be disposed of using the `dispose_byte_array` function.
ByteArray *load_fanart(PopcornFX *popcorn_fx, const MediaItemC *media);

/// Load the image data from the given URL.
///
/// If image data is available for the provided URL, it is returned as a ByteArray.
/// Otherwise, a null pointer is returned when the data couldn't be loaded.
///
/// # Arguments
///
/// * popcorn_fx - a mutable reference to a PopcornFX instance.
/// * url - a pointer to a null-terminated C string that contains the URL from which to load the image data.
///
/// # Safety
///
/// This function should only be called from C code, and the returned byte array should be disposed of using the dispose_byte_array function.
ByteArray *load_image(PopcornFX *popcorn_fx, char *url);

/// Load the poster image data for the given media item.
///
/// If poster image data is available for the media item, it is returned as a `ByteArray`.
/// Otherwise, a placeholder `ByteArray` containing the default poster holder image data is returned.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to a `PopcornFX` instance.
/// * `media` - a reference to a `MediaItemC` object that represents the media item to load.
///
/// # Safety
///
/// This function should only be called from C code, and the returned byte array should be disposed of using the `dispose_byte_array` function.
ByteArray *load_poster(PopcornFX *popcorn_fx, const MediaItemC *media);

/// Cancels the current media loading process initiated by the `MediaLoader`.
///
/// # Arguments
///
/// * `instance` - A mutable reference to the `PopcornFX` instance.
void loader_cancel(PopcornFX *instance, LoadingHandleC handle);

/// Load a media item using the media loader from a C-compatible URL.
///
/// This function takes a mutable reference to a `PopcornFX` instance and a C-compatible string (`*mut c_char`) representing the URL of the media item to load.
/// It uses the media loader to load the media item asynchronously and returns a handle (represented as a `LoadingHandleC`) for the loading process.
///
/// # Arguments
///
/// * `instance` - A mutable reference to the `PopcornFX` instance.
/// * `url` - A C-compatible string representing the URL of the media item to load.
///
/// # Returns
///
/// A `LoadingHandleC` representing the loading process associated with the loaded item.
LoadingHandleC loader_load(PopcornFX *instance, char *url);

/// Logs a message sent over FFI using the Rust logger.
///
/// # Arguments
///
/// * `message` - A pointer to the null-terminated C string containing the log message to be logged.
/// * `level` - The log level of the message. Determines the verbosity of the message and how it will be formatted by the Rust logger.
void log(char *target, char *message, LogLevel level);

/// Retrieve the available subtitles for the given [MovieDetailsC].
///
/// This function takes a reference to the `PopcornFX` instance and a reference to a `MovieDetailsC`.
/// It returns a reference to `SubtitleInfoSet` containing the available subtitles for the movie,
/// or a null pointer (`ptr::null_mut()`) on failure.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the `PopcornFX` instance.
/// * `movie` - A reference to the `MovieDetailsC` for which subtitles are to be retrieved.
///
/// # Returns
///
/// A pointer to the `SubtitleInfoSet` containing the available subtitles, or a null pointer on failure.
/// <i>The returned reference should be managed by the caller.</i>
SubtitleInfoSet *movie_subtitles(PopcornFX *popcorn_fx, const MovieDetailsC *movie);

/// Create a new PopcornFX instance.
/// The caller will become responsible for managing the memory of the struct.
/// The instance can be safely deleted by using [dispose_popcorn_fx].
PopcornFX *new_popcorn_fx(int32_t len, char **args);

/// Play the next item in the playlist from C.
///
/// This function is exposed as a C-compatible function and is intended to be called from C or other languages.
/// It takes a mutable reference to a `PopcornFX` instance and attempts to start playback of the next item in the playlist managed by the `PlaylistManager`.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
///
/// # Returns
///
/// A raw pointer to an `i64` representing the handle of the playlist item if playback was successfully started;
/// otherwise, a null pointer if there are no more items to play or if an error occurred during playback initiation.
const int64_t *play_next_playlist_item(PopcornFX *popcorn_fx);

/// Play a playlist from C by converting it to the Rust data structure and starting playback asynchronously.
///
/// This function takes a mutable reference to a `PopcornFX` instance and a C-compatible array of `PlaylistItemC` items.
/// It converts the C array into a Rust `Playlist` and starts playback asynchronously using the playlist manager.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
/// * `playlist` - A C-compatible array of `PlaylistItemC` items representing the playlist to play.
///
/// # Returns
///
/// If the playlist playback is successfully started, a pointer to the internal playlist handle is returned.
/// Otherwise, if an error occurs or the playlist is empty, a null pointer is returned.
const int64_t *play_playlist(PopcornFX *popcorn_fx, const CArray<PlaylistItemC> *playlist_c);

/// Retrieve a pointer to a `PlayerC` instance by its unique identifier (ID) from the PopcornFX player manager.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `popcorn_fx` and `player_id` pointers.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `player_id` - A pointer to a null-terminated C string representing the player's unique identifier (ID).
///
/// # Returns
///
/// Returns a pointer to a `PlayerC` instance representing the player if found, or a null pointer if no player with the given ID exists.
PlayerC *player_by_id(PopcornFX *popcorn_fx, char *player_id);

/// Pauses the player associated with the given `PlayerWrapperC` instance.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++),
/// and the caller is responsible for ensuring the safety of the provided `player` pointer.
///
/// # Arguments
///
/// * `player` - A mutable reference to a `PlayerWrapperC` instance.
void player_pause(PlayerWrapperC *player);

/// Retrieves a pointer to a `PlayerWrapperC` instance by its unique identifier (ID) from the PopcornFX player manager.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `popcorn_fx` and `player_id` pointers.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `player_id` - A pointer to a null-terminated C string representing the player's unique identifier (ID).
///
/// # Returns
///
/// Returns a pointer to a `PlayerWrapperC` instance representing the player if found, or a null pointer if no player with the given ID exists.
PlayerWrapperC *player_pointer_by_id(PopcornFX *popcorn_fx, char *player_id);

/// Resumes the player associated with the given `PlayerWrapperC` instance.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++),
/// and the caller is responsible for ensuring the safety of the provided `player` pointer.
///
/// # Arguments
///
/// * `player` - A mutable reference to a `PlayerWrapperC` instance.
void player_resume(PlayerWrapperC *player);

/// Seeks the player associated with the given `PlayerWrapperC` instance to the specified time position.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++),
/// and the caller is responsible for ensuring the safety of the provided `player` pointer.
///
/// # Arguments
///
/// * `player` - A mutable reference to a `PlayerWrapperC` instance.
/// * `time` - The time position to seek to, in milliseconds.
void player_seek(PlayerWrapperC *player, uint64_t time);

/// Stops the player associated with the given `PlayerWrapperC` instance.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++),
/// and the caller is responsible for ensuring the safety of the provided `player` pointer.
///
/// # Arguments
///
/// * `player` - A mutable reference to a `PlayerWrapperC` instance.
void player_stop(PlayerWrapperC *player);

/// Retrieve a pointer to a `PlayerSet` containing information about all players managed by PopcornFX.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `popcorn_fx` pointer.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
///
/// # Returns
///
/// Returns a pointer to a `PlayerSet` containing information about all players managed by PopcornFX.
PlayerSet *players(PopcornFX *popcorn_fx);

/// Retrieves the playlist from PopcornFX.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
///
/// # Returns
///
/// A CArray of PlaylistItemC representing the playlist.
CArray<PlaylistItemC> *playlist(PopcornFX *popcorn_fx);

/// Retrieve the default poster (placeholder) image data as a C compatible byte array.
///
/// This function returns a pointer to a `ByteArray` struct that contains the data for the default poster placeholder image.
/// The default poster placeholder image is typically used as a fallback when a poster image is not available for a media item.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the `PopcornFX` instance.
///
/// # Returns
///
/// A pointer to a `ByteArray` struct containing the default poster holder image data.
///
/// # Safety
///
/// This function should only be called from C code, and the returned byte array should be disposed of using the `dispose_byte_array` function.
ByteArray *poster_placeholder(PopcornFX *popcorn_fx);

/// Publish a new application event over the FFI layer.
/// This will invoke the [popcorn_fx_core::core::event::EventPublisher] publisher on the backend.
///
/// _Please keep in mind that the consumption of the event chain is not communicated over the FFI layer_
void publish_event(PopcornFX *popcorn_fx, EventC event);

/// Register an event callback with the PopcornFX event publisher.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `popcorn_fx` and `callback` pointers.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `callback` - A C-compatible function pointer representing the callback to be registered.
void register_event_callback(PopcornFX *popcorn_fx, EventCCallback callback);

/// Register a new callback listener for favorite events.
void register_favorites_event_callback(PopcornFX *popcorn_fx, void (*callback)(FavoriteEventC));

/// Register a fullscreen callback function.
///
/// This function registers a new fullscreen callback function to handle fullscreen events within the PopcornFX instance.
///
/// # Arguments
///
/// * `instance` - A mutable reference to the `PopcornFX` instance.
/// * `callback` - The fullscreen callback function to be registered.
void register_fullscreen_callback(PopcornFX *instance, FullscreenCallback callback);

/// Register a callback function to check if the application is in fullscreen mode.
///
/// This function registers a new callback function to check the fullscreen state within the PopcornFX instance.
///
/// # Arguments
///
/// * `instance` - A mutable reference to the `PopcornFX` instance.
/// * `callback` - The callback function to be registered for checking the fullscreen state.
void register_is_fullscreen_callback(PopcornFX *instance, IsFullscreenCallback callback);

/// Register a loader event callback to receive loader state change events.
///
/// This function registers a callback function to receive loader state change events from the
/// PopcornFX instance. When a loader state change event occurs, the provided callback will be invoked.
///
/// # Arguments
///
/// * `instance` - A mutable reference to the PopcornFX instance to register the callback with.
/// * `callback` - A C-compatible callback function that will be invoked when loader state change events occur.
void register_loader_callback(PopcornFX *instance, LoaderEventCallback callback);

/// Register a new callback listener for the system playback controls.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to a `PopcornFX` instance.
/// * `callback` - a callback function pointer of type `PlaybackControlsCallbackC`.
///
/// # Safety
///
/// This function should only be called from C code and the callback function should be implemented in C as well.
/// The `callback` function pointer should point to a valid C function that can receive a `PlaybackControlsEventC` parameter and return nothing.
/// The callback function will be invoked whenever a playback control event occurs in the system.
void register_playback_controls(PopcornFX *popcorn_fx, PlaybackControlsCallbackC callback);

/// Register a player with the PopcornFX player manager.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `popcorn_fx` and `player` pointers.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `player` - A `PlayerRegistrationC` instance to be registered with the player manager.
///
/// # Notes
///
/// This function registers a player with the PopcornFX player manager using the provided `PlayerC` instance.
/// It logs an info message if the registration is successful and a warning message if registration fails.
void register_player(PopcornFX *popcorn_fx, PlayerRegistrationC player);

/// Register a callback function to be notified of player manager events.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `popcorn_fx` pointer.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `callback` - A C-compatible callback function that will be invoked when player manager events occur.
void register_player_callback(PopcornFX *popcorn_fx, PlayerManagerEventCallback callback);

/// Registers a C-compatible callback function to receive playlist manager events.
///
/// This function is exposed as a C-compatible function and is intended to be called from C or other languages.
/// It takes a mutable reference to a `PopcornFX` instance and a C-compatible callback function as arguments.
///
/// The function registers the provided callback function with the `PlaylistManager` from the `PopcornFX` instance.
/// When a playlist manager event occurs, the callback function is invoked with the corresponding C-compatible event data.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with C-compatible code and dereferences raw pointers.
/// Users of this function should ensure that they provide a valid `PopcornFX` instance and a valid `PlaylistManagerCallbackC`.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the `PopcornFX` instance.
/// * `callback` - The C-compatible callback function to be registered.
void register_playlist_manager_callback(PopcornFX *popcorn_fx, PlaylistManagerCallbackC callback);

/// Register a new callback for all setting events.
void register_settings_callback(PopcornFX *popcorn_fx, ApplicationConfigCallbackC callback);

/// Register a new callback for subtitle events.
///
/// # Safety
///
/// This function should only be called from C code.
/// The `popcorn_fx` pointer must be valid and properly initialized.
/// The `callback` function pointer should point to a valid C function that can receive a `SubtitleEventC` parameter and return nothing.
/// The callback function will be invoked whenever a subtitle event occurs in the system.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `callback` - A function pointer to the C callback function.
void register_subtitle_callback(PopcornFX *popcorn_fx, SubtitleCallbackC callback);

/// Registers a callback function to handle authorization URI openings from C code.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `callback` - The callback function to be registered.
void register_tracking_authorization_open(PopcornFX *popcorn_fx, AuthorizationOpenC callback);

/// Registers a callback function to handle tracking provider events from C code.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `callback` - The callback function to be registered.
void register_tracking_provider_callback(PopcornFX *popcorn_fx, TrackingEventCCallback callback);

/// Register a new callback for update events.
///
/// This function registers a new callback listener for update events in the PopcornFX application.
/// The `callback` argument should be a C-compatible function that will be invoked when an update event occurs.
///
/// The `callback` function should take a single argument of type `UpdateEventC` and return nothing.
/// The `UpdateEventC` type is a C-compatible version of the `UpdateEvent` enum used internally by the PopcornFX updater.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to a `PopcornFX` instance.
/// * `callback` - a C-compatible function that will be invoked when an update event occurs.
///
/// # Safety
///
/// This function should only be called from C code, and the provided `callback` function should be a valid C function pointer.
void register_update_callback(PopcornFX *popcorn_fx, UpdateCallbackC callback);

/// Register a new callback listener for watched events.
void register_watched_event_callback(PopcornFX *popcorn_fx, void (*callback)(WatchedEventC));

/// Reload the settings of the application.
void reload_settings(PopcornFX *popcorn_fx);

/// Remove the media item from favorites.
void remove_from_favorites(PopcornFX *popcorn_fx, const MediaItemC *favorite);

/// Remove the given media item from the watched list.
void remove_from_watched(PopcornFX *popcorn_fx, const MediaItemC *watchable);

/// Remove a player with the specified ID from the PopcornFX player manager.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `popcorn_fx` and `player_id` pointers.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `player_id` - A pointer to a null-terminated C string representing the player's unique identifier (ID).
///
/// # Notes
///
/// This function removes a player with the specified ID from the PopcornFX player manager.
/// It converts the `player_id` C string to a Rust String and logs a trace message to indicate the removal.
void remove_player(PopcornFX *popcorn_fx, char *player_id);

/// Reset all available api stats for the movie api.
/// This will make all disabled api's available again.
void reset_movie_apis(PopcornFX *popcorn_fx);

/// Reset all available api stats for the movie api.
/// This will make all disabled api's available again.
void reset_show_apis(PopcornFX *popcorn_fx);

/// Reset the current preferred subtitle configuration.
/// This will remove any selected [SubtitleInfo] or custom subtitle file.
void reset_subtitle(PopcornFX *popcorn_fx);

/// Retrieve all favorites of the user.
///
/// It will return an array of favorites on success, else [ptr::null_mut].
VecFavoritesC *retrieve_all_favorites(PopcornFX *popcorn_fx);

/// Retrieve all watched media item id's.
///
/// It returns an array of watched id's.
StringArray retrieve_all_watched(PopcornFX *popcorn_fx);

/// Retrieves available favorites from a PopcornFX instance.
///
/// This function retrieves favorites from the provided `popcorn_fx` instance,
/// filtering them based on the specified `genre`, `sort_by`, `keywords`, and `page`.
///
/// # Safety
///
/// This function is marked as unsafe due to potential undefined behavior caused by
/// invalid pointers or memory access when interacting with C code.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a PopcornFX instance.
/// * `genre` - A pointer to a GenreC struct, representing the genre filter.
/// * `sort_by` - A pointer to a SortByC struct, representing the sorting criteria.
/// * `keywords` - A pointer to a C-style string containing search keywords.
/// * `page` - The page number for pagination.
///
/// # Returns
///
/// If successful, returns a pointer to a VecFavoritesC struct containing the retrieved favorites.
/// Returns a null pointer if an error occurs during the retrieval process.
VecFavoritesC *retrieve_available_favorites(PopcornFX *popcorn_fx, const GenreC *genre, const SortByC *sort_by, char *keywords, uint32_t page);

/// Retrieve the available movies for the given criteria.
///
/// It returns the [VecMovieC] reference on success, else [ptr::null_mut].
MediaSetResult retrieve_available_movies(PopcornFX *popcorn_fx, const GenreC *genre, const SortByC *sort_by, char *keywords, uint32_t page);

/// Retrieve the available [ShowOverviewC] items for the given criteria.
///
/// It returns an array of [ShowOverviewC] items on success, else a [ptr::null_mut].
MediaSetResult retrieve_available_shows(PopcornFX *popcorn_fx, const GenreC *genre, const SortByC *sort_by, char *keywords, uint32_t page);

/// Retrieve the details of a favorite item on the given IMDB ID.
/// The details contain all information about the media item.
///
/// It returns the [MediaItemC] on success, else a [ptr::null_mut].
MediaResult retrieve_media_details(PopcornFX *popcorn_fx, const MediaItemC *media);

/// Retrieve the array of available genres for the given provider.
///
/// It returns an empty list when the provider name doesn't exist.
StringArray *retrieve_provider_genres(PopcornFX *popcorn_fx, char *name);

/// Retrieve the array of available sorts for the given provider.
///
/// It returns an empty list when the provider name doesn't exist.
StringArray *retrieve_provider_sort_by(PopcornFX *popcorn_fx, char *name);

/// Retrieves the current subtitle preference from PopcornFX.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
///
/// # Returns
///
/// A pointer to a `SubtitlePreference` instance.
///
SubtitlePreference *retrieve_subtitle_preference(PopcornFX *popcorn_fx);

/// Retrieve all watched movie id's.
///
/// It returns an array of watched movie id's.
StringArray *retrieve_watched_movies(PopcornFX *popcorn_fx);

/// Retrieve all watched show media id's.
///
/// It returns  an array of watched show id's.
StringArray *retrieve_watched_shows(PopcornFX *popcorn_fx);

/// Selects the default subtitle from the given list of subtitles provided in C-compatible form.
///
/// This function retrieves the default subtitle selection from the provided list of subtitles,
/// converts the selected subtitle back into a C-compatible format, and returns a pointer to it.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
/// * `subtitles_ptr` - Pointer to the array of subtitles in C-compatible form.
/// * `len` - The length of the subtitles array.
///
/// # Returns
///
/// A pointer to the selected default subtitle in C-compatible form.
SubtitleInfoC *select_or_default_subtitle(PopcornFX *popcorn_fx, SubtitleInfoSet *set);

/// Set the active player in the PopcornFX player manager.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `popcorn_fx` and `player_id` pointers.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `player_id` - A pointer to a null-terminated C string representing the player's unique identifier (ID).
void set_active_player(PopcornFX *popcorn_fx, char *player_id);

/// Stop the playback of the current playlist from C.
///
/// This function is exposed as a C-compatible function and is intended to be called from C or other languages.
/// It takes a mutable reference to a `PopcornFX` instance and stops the playback of the currently playing item in the playlist.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the `PopcornFX` instance.
void stop_playlist(PopcornFX *popcorn_fx);

/// Retrieve a special [SubtitleInfo::custom] instance of the application.
///
/// # Safety
///
/// This function should only be called from C code.
///
/// # Returns
///
/// A pointer to a `SubtitleInfoC` instance representing "custom".
SubtitleInfoC *subtitle_custom();

/// Retrieve a special [SubtitleInfo::none] instance of the application.
///
/// # Safety
///
/// This function should only be called from C code.
///
/// # Returns
///
/// A pointer to a `SubtitleInfoC` instance representing "none".
SubtitleInfoC *subtitle_none();

/// Add the given magnet info to the torrent collection.
void torrent_collection_add(PopcornFX *popcorn_fx, char *name, char *magnet_uri);

/// Retrieve all stored magnets from the torrent collection.
/// It returns the set on success, else [ptr::null_mut].
TorrentCollectionSet *torrent_collection_all(PopcornFX *popcorn_fx);

/// Verify if the given magnet uri has already been stored.
bool torrent_collection_is_stored(PopcornFX *popcorn_fx, char *magnet_uri);

/// Remove the given magnet uri from the torrent collection.
void torrent_collection_remove(PopcornFX *popcorn_fx, char *magnet_uri);

/// Calculates the health of a torrent based on its magnet link.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
/// * `uri` - The magnet link of the torrent.
///
/// # Returns
///
/// Returns the health of the torrent.
ResultC<TorrentHealth, TorrentErrorC> torrent_health_from_uri(PopcornFX *popcorn_fx, const char *uri);

/// Initiates the authorization process with the tracking provider.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
void tracking_authorize(PopcornFX *popcorn_fx);

/// Disconnects from the tracking provider.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
void tracking_disconnect(PopcornFX *popcorn_fx);

/// Checks if the current tracking provider is authorized.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
///
/// # Returns
///
/// Returns `true` if the tracking provider is authorized, otherwise `false`.
bool tracking_is_authorized(PopcornFX *popcorn_fx);

/// Update the playback settings with the new value.
void update_playback_settings(PopcornFX *popcorn_fx, PlaybackSettingsC settings);

/// Update the server settings with the new value.
void update_server_settings(PopcornFX *popcorn_fx, ServerSettingsC settings);

/// Retrieve the current update state of the application.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to a `PopcornFX` instance.
///
/// # Returns
///
/// The current update state of the application as a [UpdateStateC] value.
UpdateStateC update_state(PopcornFX *popcorn_fx);

/// Updates the subtitle preference for PopcornFX.
///
/// # Arguments
///
/// * `popcorn_fx` - Mutable reference to the PopcornFX instance.
/// * `preference` - The new subtitle preference to set.
void update_subtitle_preference(PopcornFX *popcorn_fx, const SubtitlePreference *preference);

/// Update the subtitle settings with the new value.
void update_subtitle_settings(PopcornFX *popcorn_fx, SubtitleSettingsC subtitle_settings);

/// Update the torrent settings with the new value.
void update_torrent_settings(PopcornFX *popcorn_fx, TorrentSettingsC torrent_settings);

/// Update the ui settings with the new value.
void update_ui_settings(PopcornFX *popcorn_fx, UiSettingsC settings);

/// Retrieve the version of Popcorn FX.
char *version();

/// Retrieve the latest release version information from the update channel.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to a `PopcornFX` instance.
VersionInfoC *version_info(PopcornFX *popcorn_fx);

}  // extern "C"
