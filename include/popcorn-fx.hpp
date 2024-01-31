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

enum class LoadingState : int32_t {
  Initializing,
  Starting,
  RetrievingSubtitles,
  DownloadingSubtitle,
  Connecting,
  Downloading,
  DownloadFinished,
  Ready,
  Playing,
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

/// The state of a [Torrent] which is represented as a [i32].
/// This state is abi compatible to be used over [std::ffi].
enum class TorrentState : int32_t {
  /// The initial phase of the torrent in which it's still being created.
  /// This is the state where the metadata of the torrent is retrieved.
  Creating = 0,
  /// The torrent is ready to be downloaded (metadata is available).
  Ready = 1,
  /// The download of the torrent is starting.
  Starting = 2,
  /// The torrent is being downloaded.
  Downloading = 3,
  /// The torrent download has been paused.
  Paused = 4,
  /// The torrent download has completed.
  Completed = 5,
  /// The torrent encountered an error and cannot be downloaded.
  Error = -1,
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

/// A C-compatible struct representing a player.
struct PlayerC {
  /// A pointer to a null-terminated C string representing the player's unique identifier (ID).
  const char *id;
  /// A pointer to a null-terminated C string representing the name of the player.
  const char *name;
  /// A pointer to a null-terminated C string representing the description of the player.
  const char *description;
  /// A pointer to a `ByteArray` struct representing the graphic resource associated with the player.
  ///
  /// This field can be a null pointer if no graphic resource is associated with the player.
  ByteArray *graphic_resource;
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
  const char *poster;
  const char *fanart;
  const char *banner;
};

struct MovieOverviewC {
  const char *title;
  const char *imdb_id;
  const char *year;
  RatingC *rating;
  ImagesC images;
};

/// A C-compatible struct representing torrent file information.
struct TorrentFileInfoC {
  /// A pointer to a null-terminated C string representing the filename.
  const char *filename;
  /// A pointer to a null-terminated C string representing the file path.
  const char *file_path;
  /// The size of the file in bytes.
  int64_t file_size;
  /// The index of the file.
  int32_t file_index;
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
  /// A pointer to a null-terminated C string representing the torrent URL.
  const char *url;
  /// A pointer to a null-terminated C string representing the torrent provider.
  const char *provider;
  /// A pointer to a null-terminated C string representing the torrent source.
  const char *source;
  /// A pointer to a null-terminated C string representing the torrent title.
  const char *title;
  /// A pointer to a null-terminated C string representing the torrent quality.
  const char *quality;
  /// The number of seeders for the torrent.
  uint32_t seed;
  /// The number of peers for the torrent.
  uint32_t peer;
  /// A pointer to a null-terminated C string representing the torrent size in bytes.
  const char *size;
  /// A pointer to a null-terminated C string representing the torrent filesize in human-readable format.
  const char *filesize;
  /// A pointer to a null-terminated C string representing the selected file within the torrent collection.
  const char *file;
};

/// A C-compatible struct representing torrent information.
struct TorrentInfoC {
  /// A pointer to a null-terminated C string representing the name of the torrent.
  const char *name;
  /// A pointer to a null-terminated C string representing the directory name of the torrent.
  const char *directory_name;
  /// The total number of files in the torrent.
  int32_t total_files;
  /// A set of `TorrentFileInfoC` structs representing individual files within the torrent.
  CArray<TorrentFileInfoC> files;
};

struct TorrentQualityC {
  const char *quality;
  TorrentInfoC torrent;
};

struct TorrentEntryC {
  const char *language;
  TorrentQualityC *qualities;
  int32_t len;
};

/// The C compatible [MovieDetails] representation
///
/// Use the [MovieDetails::from] to convert the C instance back to a rust struct.
struct MovieDetailsC {
  const char *title;
  const char *imdb_id;
  const char *year;
  RatingC *rating;
  ImagesC images;
  const char *synopsis;
  int32_t runtime;
  const char *trailer;
  const char **genres;
  int32_t genres_len;
  TorrentEntryC *torrents;
  int32_t torrents_len;
};

struct ShowOverviewC {
  const char *imdb_id;
  const char *tvdb_id;
  const char *title;
  const char *year;
  int32_t num_seasons;
  ImagesC images;
  RatingC *rating;
};

/// The C compatible [Episode] media information.
struct EpisodeC {
  int32_t season;
  int32_t episode;
  int64_t first_aired;
  const char *title;
  const char *synopsis;
  const char *tvdb_id;
  const char *thumb;
  TorrentQualityC *torrents;
  int32_t len;
};

struct ShowDetailsC {
  const char *imdb_id;
  const char *tvdb_id;
  const char *title;
  const char *year;
  int32_t num_seasons;
  ImagesC images;
  RatingC *rating;
  const char *synopsis;
  int32_t runtime;
  const char *status;
  const char **genres;
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
  const char *directory;
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
  const char *directory;
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
  const char *default_language;
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
  const char *api_server;
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
};

/// The C compatible [SubtitleFile] representation.
struct SubtitleFileC {
  int32_t file_id;
  const char *name;
  const char *url;
  float score;
  int32_t downloads;
  const int32_t *quality;
};

/// The C compatible [SubtitleInfo] representation.
struct SubtitleInfoC {
  /// The IMDB ID if known, this can be [ptr::null]
  const char *imdb_id;
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

struct PlaylistItemC {
  const char *url;
  const char *title;
  const char *thumb;
  const char *quality;
  MediaItemC *parent_media;
  MediaItemC *media;
  uint64_t *auto_resume_timestamp;
  bool subtitles_enabled;
};

struct StyledTextC {
  const char *text;
  bool italic;
  bool bold;
  bool underline;
};

struct SubtitleLineC {
  StyledTextC *texts;
  int32_t len;
};

struct SubtitleCueC {
  const char *id;
  uint64_t start_time;
  uint64_t end_time;
  SubtitleLineC *lines;
  int32_t number_of_lines;
};

/// The parsed subtitle representation for C.
/// It contains the data of a subtitle file that can be displayed.
struct SubtitleC {
  /// The filepath that has been parsed
  const char *file;
  /// The info of the parsed subtitle if available, else [ptr::null_mut]
  SubtitleInfoC *info;
  /// The parsed cues from the subtitle file
  SubtitleCueC *cues;
  /// The total number of cue elements
  int32_t len;
};

/// The C compatible struct for [MagnetInfo].
struct MagnetInfoC {
  /// The name of the magnet
  const char *name;
  /// The magnet uri to the torrent
  const char *magnet_uri;
};

/// The collection of stored magnets.
/// It contains the C compatible information for [std::ffi].
struct TorrentCollectionSet {
  /// The array of magnets
  MagnetInfoC *magnets;
  /// The length of the array
  int32_t len;
};

/// The subtitle matcher C compatible struct.
/// It contains the information which should be matched when selecting a subtitle file to load.
struct SubtitleMatcherC {
  /// The nullable name of the media item.
  const char *name;
  /// The nullable quality of the media item.
  /// This can be represented as `720p` or `720`.
  const char *quality;
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

struct PlayerSet {
  PlayerC *players;
  int32_t len;
};

/// A C-compatible struct representing a player change event.
struct PlayerChangedEventC {
  /// The (nullable) old player id
  const char *old_player_id;
  /// The new player id
  const char *new_player_id;
  /// The new player name
  const char *new_player_name;
};

struct PlayerStartedEventC {
  const char *url;
  const char *title;
  const char *thumbnail;
  const char *quality;
  uint64_t *auto_resume_timestamp;
  bool subtitles_enabled;
};

/// The player stopped event which indicates a video playback has been stopped.
/// It contains the last known information of the video playback right before it was stopped.
struct PlayerStoppedEventC {
  /// The playback url that was being played
  const char *url;
  /// The last known video time of the player in millis
  const int64_t *time;
  /// The duration of the video playback in millis
  const int64_t *duration;
  /// The optional media item that was being played
  MediaItemC *media;
};

/// The C compatible [Event] representation.
struct EventC {
  enum class Tag {
    /// Invoked when the player is changed
    /// 1ste argument is the new player id, 2nd argument is the new player name
    PlayerChanged,
    PlayerStarted,
    /// Invoked when the player is being stopped
    PlayerStopped,
    /// Invoked when the playback state is changed
    PlaybackStateChanged,
    /// Invoked when the watch state of an item is changed
    WatchStateChanged,
    LoadingStarted,
    LoadingCompleted,
  };

  struct PlayerChanged_Body {
    PlayerChangedEventC _0;
  };

  struct PlayerStarted_Body {
    PlayerStartedEventC _0;
  };

  struct PlayerStopped_Body {
    PlayerStoppedEventC _0;
  };

  struct PlaybackStateChanged_Body {
    PlaybackState _0;
  };

  struct WatchStateChanged_Body {
    const char *_0;
    bool _1;
  };

  Tag tag;
  union {
    PlayerChanged_Body player_changed;
    PlayerStarted_Body player_started;
    PlayerStopped_Body player_stopped;
    PlaybackStateChanged_Body playback_state_changed;
    WatchStateChanged_Body watch_state_changed;
  };
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
    /// * `*const c_char`   - The imdb id of the media item that changed.
    /// * `bool`            - The new like state of the media item.
    LikedStateChanged,
  };

  struct LikedStateChanged_Body {
    const char *_0;
    bool _1;
  };

  Tag tag;
  union {
    LikedStateChanged_Body liked_state_changed;
  };
};

/// A C-compatible struct representing the event when loading starts.
/// A C-compatible struct representing the event when loading starts.
struct LoadingStartedEventC {
  /// The URL of the media being loaded.
  const char *url;
  /// The title or name of the media being loaded.
  const char *title;
  /// The URL of a thumbnail image associated with the media, or `ptr::null()` if not available.
  const char *thumbnail;
  /// The URL of a background image associated with the media, or `ptr::null()` if not available.
  const char *background;
  /// The quality or resolution information of the media, or `ptr::null()` if not available.
  const char *quality;
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
    Cancelled,
  };

  struct ParseError_Body {
    const char *_0;
  };

  struct TorrentError_Body {
    const char *_0;
  };

  struct MediaError_Body {
    const char *_0;
  };

  struct TimeoutError_Body {
    const char *_0;
  };

  Tag tag;
  union {
    ParseError_Body parse_error;
    TorrentError_Body torrent_error;
    MediaError_Body media_error;
    TimeoutError_Body timeout_error;
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

/// A C-compatible callback function type for loader events.
using LoaderEventCallback = void(*)(LoaderEventC);

/// The C compatible callback for playback control events.
using PlaybackControlsCallbackC = void(*)(PlaybackControlEvent);

struct PlayRequestC {
  const char *url;
  const char *title;
  const char *thumb;
  uint64_t *auto_resume_timestamp;
  bool subtitles_enabled;
};

/// A C-compatible callback function type for player play events.
using PlayerPlayCallback = void(*)(PlayRequestC);

/// A C-compatible callback function type for player stop events.
using PlayerStopCallback = void(*)();

/// A C-compatible struct representing player registration information.
struct PlayerRegistrationC {
  /// A pointer to a null-terminated C string representing the player's unique identifier (ID).
  const char *id;
  /// A pointer to a null-terminated C string representing the name of the player.
  const char *name;
  /// A pointer to a null-terminated C string representing the description of the player.
  const char *description;
  /// A pointer to a `ByteArray` struct representing the graphic resource associated with the player.
  ///
  /// This field can be a null pointer if no graphic resource is associated with the player.
  ByteArray *graphic_resource;
  /// The state of the player.
  PlayerState state;
  /// Indicates whether embedded playback is supported by the player.
  bool embedded_playback_supported;
  /// A callback function pointer for the "play" action.
  PlayerPlayCallback play_callback;
  /// A callback function pointer for the "stop" action.
  PlayerStopCallback stop_callback;
};

struct PlayerManagerEventC {
  enum class Tag {
    ActivePlayerChanged,
    PlayersChanged,
    PlayerDurationChanged,
    PlayerTimeChanged,
    PlayerStateChanged,
  };

  struct ActivePlayerChanged_Body {
    PlayerChangedEventC _0;
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
    PlayerDurationChanged_Body player_duration_changed;
    PlayerTimeChanged_Body player_time_changed;
    PlayerStateChanged_Body player_state_changed;
  };
};

/// A C-compatible callback function type for player manager events.
using PlayerManagerEventCallback = void(*)(PlayerManagerEventC);

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

  Tag tag;
  union {
    SubtitleSettingsChanged_Body subtitle_settings_changed;
    TorrentSettingsChanged_Body torrent_settings_changed;
    UiSettingsChanged_Body ui_settings_changed;
    ServerSettingsChanged_Body server_settings_changed;
    PlaybackSettingsChanged_Body playback_settings_changed;
  };
};

/// The C callback for the setting events.
using ApplicationConfigCallbackC = void(*)(ApplicationConfigEventC);

/// The C compatible [SubtitleEvent] representation
struct SubtitleEventC {
  enum class Tag {
    SubtitleInfoChanged,
    PreferredLanguageChanged,
  };

  struct SubtitleInfoChanged_Body {
    SubtitleInfoC *_0;
  };

  struct PreferredLanguageChanged_Body {
    SubtitleLanguage _0;
  };

  Tag tag;
  union {
    SubtitleInfoChanged_Body subtitle_info_changed;
    PreferredLanguageChanged_Body preferred_language_changed;
  };
};

/// The C callback for the subtitle events.
using SubtitleCallbackC = void(*)(SubtitleEventC);

/// The C compatible representation of the application runtime information.
struct PatchInfoC {
  /// The runtime version of the application.
  const char *version;
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
    /// * `*const c_char`   - The imdb id of the media item that changed.
    /// * `bool`            - The new watched state of the media item.
    WatchedStateChanged,
  };

  struct WatchedStateChanged_Body {
    const char *_0;
    bool _1;
  };

  Tag tag;
  union {
    WatchedStateChanged_Body watched_state_changed;
  };
};

struct VecFavoritesC {
  MovieOverviewC *movies;
  int32_t movies_len;
  ShowOverviewC *shows;
  int32_t shows_len;
};

/// The C compatible string array.
/// It's mainly used for returning string arrays as result of C function calls.
struct StringArray {
  /// The string array
  const char **values;
  /// The length of the string array
  int32_t len;
};

struct GenreC {
  const char *key;
  const char *text;
};

struct SortByC {
  const char *key;
  const char *text;
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

/// The callback to verify if the given byte is available.
using HasByteCallbackC = bool(*)(int32_t, uint64_t*);

/// The callback to verify if the given piece is available.
using HasPieceCallbackC = bool(*)(uint32_t);

/// The callback to retrieve the total pieces of the torrent.
using TotalPiecesCallbackC = int32_t(*)();

/// The callback for prioritizing bytes.
using PrioritizeBytesCallbackC = void(*)(int32_t, uint64_t*);

/// The callback for prioritizing pieces.
using PrioritizePiecesCallbackC = void(*)(int32_t, uint32_t*);

/// The callback for update the torrent mode to sequential.
using SequentialModeCallbackC = void(*)();

/// The callback for retrieving the torrent state.
using TorrentStateCallbackC = TorrentState(*)();

/// The C compatible abi struct for a [Torrent].
/// This currently uses callbacks as it's a wrapper around a torrent implementation provided through C.
struct TorrentC {
  const char *handle;
  /// The filepath to the torrent file
  const char *filepath;
  HasByteCallbackC has_byte_callback;
  HasPieceCallbackC has_piece_callback;
  TotalPiecesCallbackC total_pieces;
  PrioritizeBytesCallbackC prioritize_bytes;
  PrioritizePiecesCallbackC prioritize_pieces;
  SequentialModeCallbackC sequential_mode;
  TorrentStateCallbackC torrent_state;
};

/// Type definition for a callback that resolves torrent information and starts a download.
using ResolveTorrentCallback = TorrentC(*)(TorrentFileInfoC file_info, const char *torrent_directory, bool auto_start_download);

/// Type definition for a callback that resolves torrent information.
using ResolveTorrentInfoCallback = TorrentInfoC(*)(const char *url);


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

/// Retrieve the auto-resume timestamp for the given media id and/or filename.
uint64_t *auto_resume_timestamp(PopcornFX *popcorn_fx, const char *id, const char *filename);

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

/// Disable the subtitle track on request of the user.
/// This will make the [is_subtitle_disabled] return `true`.
void disable_subtitle(PopcornFX *popcorn_fx);

/// Frees the memory allocated for the given C-compatible byte array.
///
/// This function should be called from C code in order to free memory that has been allocated by Rust.
///
/// # Safety
///
/// This function should only be called on C-compatible byte arrays that have been allocated by Rust.
void dispose_byte_array(Box<ByteArray> array);

/// Dispose the given media item from memory.
void dispose_media_item(Box<MediaItemC> media);

/// Dispose all given media items from memory.
void dispose_media_items(Box<MediaSetC> media);

/// Dispose of a playlist item.
///
/// # Arguments
///
/// * `item` - A boxed `PlaylistItemC` representing the item to be disposed of.
void dispose_playlist_item(Box<PlaylistItemC> item);

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

/// Dispose the given subtitle.
void dispose_subtitle(Box<SubtitleC> subtitle);

/// Dispose the [TorrentCollectionSet] from memory.
void dispose_torrent_collection(Box<TorrentCollectionSet> collection_set);

/// Download the given [SubtitleInfo] based on the best match according to the [SubtitleMatcher].
///
/// It returns the filepath to the subtitle on success, else [ptr::null_mut].
const char *download(PopcornFX *popcorn_fx, const SubtitleInfoC *subtitle, const SubtitleMatcherC *matcher);

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
bool is_fx_video_player_disabled(PopcornFX *popcorn_fx);

/// Verify if the application should started in kiosk mode.
/// The behavior of kiosk mode is dependant on the UI implementation and not delegated by the backend.
bool is_kiosk_mode(PopcornFX *popcorn_fx);

/// Verify if the application should be maximized on startup.
bool is_maximized(PopcornFX *popcorn_fx);

/// Verify if the given media item is liked/favorite of the user.
/// It will use the first non [ptr::null_mut] field from the [MediaItemC] struct.
///
/// It will return false if all fields in the [MediaItemC] are [ptr::null_mut].
bool is_media_liked(PopcornFX *popcorn_fx, const MediaItemC *favorite);

/// Verify if the given media item is watched by the user.
///
/// It returns true when the item is watched, else false.
bool is_media_watched(PopcornFX *popcorn_fx, const MediaItemC *watchable);

/// Verify if the application mouse should be disabled.
/// The disabling of the mouse should be implemented by the UI implementation and has no behavior on
/// the backend itself.
bool is_mouse_disabled(PopcornFX *popcorn_fx);

/// Verify if the subtitle has been disabled by the user.
///
/// It returns true when the subtitle track should be disabled, else false.
bool is_subtitle_disabled(PopcornFX *popcorn_fx);

/// Verify if the TV mode is activated for the application.
bool is_tv_mode(PopcornFX *popcorn_fx);

/// Verify if the vlc video player has been disabled.
bool is_vlc_video_player_disabled(PopcornFX *popcorn_fx);

/// Verify if the youtube video player has been disabled.
bool is_youtube_video_player_disabled(PopcornFX *popcorn_fx);

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
ByteArray *load_image(PopcornFX *popcorn_fx, const char *url);

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
void loader_cancel(PopcornFX *instance, const int64_t *handle);

/// Logs a message sent over FFI using the Rust logger.
///
/// # Arguments
///
/// * `message` - A pointer to the null-terminated C string containing the log message to be logged.
/// * `level` - The log level of the message. Determines the verbosity of the message and how it will be formatted by the Rust logger.
void log(const char *target, const char *message, LogLevel level);

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
PopcornFX *new_popcorn_fx(const char **args, int32_t len);

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
const int64_t *play_playlist(PopcornFX *popcorn_fx, CArray<PlaylistItemC> playlist);

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
PlayerC *player_by_id(PopcornFX *popcorn_fx, const char *player_id);

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
/// This will invoke the [popcorn_fx_core::core::events::EventPublisher] publisher on the backend.
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
PlayerWrapperC *register_player(PopcornFX *popcorn_fx, PlayerRegistrationC player);

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
void remove_player(PopcornFX *popcorn_fx, const char *player_id);

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

/// Retrieve all liked favorite media items.
///
/// It returns the [VecFavoritesC] holder for the array on success, else [ptr::null_mut].
VecFavoritesC *retrieve_available_favorites(PopcornFX *popcorn_fx, const GenreC *genre, const SortByC *sort_by, const char *keywords, uint32_t page);

/// Retrieve the available movies for the given criteria.
///
/// It returns the [VecMovieC] reference on success, else [ptr::null_mut].
MediaSetResult retrieve_available_movies(PopcornFX *popcorn_fx, const GenreC *genre, const SortByC *sort_by, const char *keywords, uint32_t page);

/// Retrieve the available [ShowOverviewC] items for the given criteria.
///
/// It returns an array of [ShowOverviewC] items on success, else a [ptr::null_mut].
MediaSetResult retrieve_available_shows(PopcornFX *popcorn_fx, const GenreC *genre, const SortByC *sort_by, const char *keywords, uint32_t page);

/// Retrieve the details of a favorite item on the given IMDB ID.
/// The details contain all information about the media item.
///
/// It returns the [MediaItemC] on success, else a [ptr::null_mut].
MediaResult retrieve_media_details(PopcornFX *popcorn_fx, const MediaItemC *media);

/// Retrieve the preferred subtitle instance for the next [Media] item playback.
///
/// It returns the [SubtitleInfoC] when present, else [ptr::null_mut].
SubtitleInfoC *retrieve_preferred_subtitle(PopcornFX *popcorn_fx);

/// Retrieve the preferred subtitle language for the next [Media] item playback.
///
/// It returns the preferred subtitle language.
SubtitleLanguage retrieve_preferred_subtitle_language(PopcornFX *popcorn_fx);

/// Retrieve the array of available genres for the given provider.
///
/// It returns an empty list when the provider name doesn't exist.
StringArray *retrieve_provider_genres(PopcornFX *popcorn_fx, const char *name);

/// Retrieve the array of available sorts for the given provider.
///
/// It returns an empty list when the provider name doesn't exist.
StringArray *retrieve_provider_sort_by(PopcornFX *popcorn_fx, const char *name);

/// Retrieve all watched movie id's.
///
/// It returns an array of watched movie id's.
StringArray retrieve_watched_movies(PopcornFX *popcorn_fx);

/// Retrieve all watched show media id's.
///
/// It returns  an array of watched show id's.
StringArray retrieve_watched_shows(PopcornFX *popcorn_fx);

/// Select a default subtitle language based on the settings or user interface language.
SubtitleInfoC *select_or_default_subtitle(PopcornFX *popcorn_fx, const SubtitleInfoC *subtitles_ptr, size_t len);

/// Serve the given subtitle as [SubtitleType] format.
///
/// It returns the url which hosts the [Subtitle].
const char *serve_subtitle(PopcornFX *popcorn_fx, SubtitleC subtitle, size_t output_type);

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
void set_active_player(PopcornFX *popcorn_fx, const char *player_id);

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
void torrent_collection_add(PopcornFX *popcorn_fx, const char *name, const char *magnet_uri);

/// Retrieve all stored magnets from the torrent collection.
/// It returns the set on success, else [ptr::null_mut].
TorrentCollectionSet *torrent_collection_all(PopcornFX *popcorn_fx);

/// Verify if the given magnet uri has already been stored.
bool torrent_collection_is_stored(PopcornFX *popcorn_fx, const char *magnet_uri);

/// Remove the given magnet uri from the torrent collection.
void torrent_collection_remove(PopcornFX *popcorn_fx, const char *magnet_uri);

/// Callback function for handling changes in the download status of a torrent.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
/// * `handle` - The handle to the torrent.
/// * `download_status` - The new download status of the torrent.
void torrent_download_status(PopcornFX *popcorn_fx, const char *handle, DownloadStatusC download_status);

/// Resolve the given torrent url into meta information of the torrent.
/// The url can be a magnet, http or file url to the torrent file.
void torrent_info(PopcornFX *popcorn_fx, const char *url);

/// Callback function for handling the completion of downloading a piece in a torrent.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
/// * `handle` - The handle to the torrent.
/// * `piece` - The index of the finished piece.
void torrent_piece_finished(PopcornFX *popcorn_fx, const char *handle, uint32_t piece);

/// A callback function for resolving torrents.
///
/// This function is exposed as a C-compatible function and is intended to be called from C or other languages.
/// It takes a `PopcornFX` instance and a `ResolveTorrentCallback` function as arguments.
///
/// The function registers the provided callback function with the `DefaultTorrentManager` from the `PopcornFX` instance.
/// When the callback function is invoked by the manager, it converts the arguments and the result between Rust and C types.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with C-compatible code and dereferences raw pointers.
/// Users of this function should ensure that they provide a valid `PopcornFX` instance and a valid `ResolveTorrentCallback`.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the `PopcornFX` instance.
/// * `callback` - The `ResolveTorrentCallback` function to be registered.
void torrent_resolve_callback(PopcornFX *popcorn_fx, ResolveTorrentCallback callback);

/// Registers a new C-compatible resolve torrent callback function with PopcornFX.
///
/// This function allows registering a callback that will be invoked when torrent resolution is complete.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
/// * `callback` - The C-compatible resolve torrent callback function to be registered.
///
/// # Example
///
/// ```c
/// void resolve_callback(TorrentInfoC info) {
///     // Handle resolved torrent information
/// }
///
/// // Register the C-compatible callback with PopcornFX
/// torrent_resolve_callback(popcorn_fx, resolve_callback);
/// ```
///
/// This function registers a callback that receives resolved torrent information in the form of a `TorrentInfoC` struct.
/// You can then handle this information as needed within your callback function.
///
/// Note: This function is intended for C integration with PopcornFX.
///
/// # Safety
///
/// This function performs unsafe operations, as it deals with raw C-compatible function pointers.
void torrent_resolve_info_callback(PopcornFX *popcorn_fx, ResolveTorrentInfoCallback callback);

/// Callback function for handling changes in the state of a torrent.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
/// * `handle` - The handle to the torrent.
/// * `state` - The new state of the torrent.
void torrent_state_changed(PopcornFX *popcorn_fx, const char *handle, TorrentState state);

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

/// Update the preferred subtitle for the [Media] item playback.
/// This action will reset any custom configured subtitle files.
void update_subtitle(PopcornFX *popcorn_fx, const SubtitleInfoC *subtitle);

/// Update the preferred subtitle to a custom subtitle filepath.
/// This action will reset any preferred subtitle.
void update_subtitle_custom_file(PopcornFX *popcorn_fx, const char *custom_filepath);

/// Update the subtitle settings with the new value.
void update_subtitle_settings(PopcornFX *popcorn_fx, SubtitleSettingsC subtitle_settings);

/// Update the torrent settings with the new value.
void update_torrent_settings(PopcornFX *popcorn_fx, TorrentSettingsC torrent_settings);

/// Update the ui settings with the new value.
void update_ui_settings(PopcornFX *popcorn_fx, UiSettingsC settings);

/// Retrieve the version of Popcorn FX.
const char *version();

/// Retrieve the latest release version information from the update channel.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to a `PopcornFX` instance.
VersionInfoC *version_info(PopcornFX *popcorn_fx);

} // extern "C"
