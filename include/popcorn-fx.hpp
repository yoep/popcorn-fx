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

/// The decoration to apply to the subtitle during rendering.
enum class DecorationType : int32_t {
  None = 0,
  Outline = 1,
  OpaqueBackground = 2,
  SeeThroughBackground = 3,
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
/// This describes the information of the playback state known by the player.
enum class PlaybackState : int32_t {
  /// This is the initial state and indicates that FX has no known state received from the player.
  UNKNOWN = -1,
  READY = 0,
  LOADING = 1,
  BUFFERING = 2,
  PLAYING = 3,
  PAUSED = 4,
  STOPPED = 5,
  ERROR = 6,
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
struct Arc;

template<typename T = void>
struct Box;

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

/// The C compatible struct for [TorrentStream].
struct TorrentStreamC;

/// The wrapper containing the callbacks to retrieve the actual
/// torrent information from C.
struct TorrentWrapper;

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

struct TorrentInfoC {
  const char *url;
  const char *provider;
  const char *source;
  const char *title;
  const char *quality;
  uint32_t seed;
  uint32_t peer;
  const char *size;
  const char *filesize;
  const char *file;
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
  bool auto_cleaning_enabled;
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

/// The C compatible [PlayVideo] representation.
struct PlayVideoEventC {
  /// The video playback url
  const char *url;
  /// The video playback title
  const char *title;
  /// The media playback show name
  const char *show_name;
  /// The optional video playback thumbnail
  const char *thumb;
};

/// The C compatible [Event] representation.
struct EventC {
  enum class Tag {
    /// Invoked when the player is being stopped
    PlayerStopped,
    /// Invoked when a new video playback is started
    PlayVideo,
    /// Invoked when the playback state is changed
    PlaybackStateChanged,
  };

  struct PlayerStopped_Body {
    PlayerStoppedEventC _0;
  };

  struct PlayVideo_Body {
    PlayVideoEventC _0;
  };

  struct PlaybackStateChanged_Body {
    PlaybackState _0;
  };

  Tag tag;
  union {
    PlayerStopped_Body player_stopped;
    PlayVideo_Body play_video;
    PlaybackStateChanged_Body playback_state_changed;
  };
};

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

/// The C compatible callback for playback control events.
using PlaybackControlsCallbackC = void(*)(PlaybackControlEvent);

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

/// The C abi compatible torrent stream event.
struct TorrentStreamEventC {
  enum class Tag {
    StateChanged,
  };

  struct StateChanged_Body {
    TorrentStreamState _0;
  };

  Tag tag;
  union {
    StateChanged_Body state_changed;
  };
};

/// The C compatible representation of the application runtime information.
struct RuntimeInfoC {
  /// The runtime version of the application.
  const char *version;
};

/// The C compatible representation of version information from the update channel.
struct VersionInfoC {
  /// The latest release version on the update channel.
  const char *version;
  /// The runtime version of the application.
  RuntimeInfoC runtime;
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

/// The wrapper communication between rust and C.
/// This is a temp wrapper which will be replaced in the future.
struct TorrentWrapperC {
  Arc<TorrentWrapper> wrapper;
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


extern "C" {

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

/// Retrieve the default options available for the subtitles.
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

/// Delete the PopcornFX instance, given as a [ptr], in a safe way.
/// All data within the instance will be deleted from memory making the instance unusable.
/// This means that the original pointer will become invalid.
void dispose_popcorn_fx(Box<PopcornFX>);

/// Dispose the given subtitle.
void dispose_subtitle(Box<SubtitleC> subtitle);

/// Dispose the [TorrentCollectionSet] from memory.
void dispose_torrent_collection(Box<TorrentCollectionSet> collection_set);

/// Dispose the torrent stream.
/// Make sure [stop_stream] has been called before dropping the instance.
void dispose_torrent_stream(Box<TorrentStreamC> stream);

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

/// Retrieve the given subtitles for the given episode
SubtitleInfoSet *episode_subtitles(PopcornFX *popcorn_fx, const ShowDetailsC *show, const EpisodeC *episode);

/// Retrieve the available subtitles for the given filename
SubtitleInfoSet *filename_subtitles(PopcornFX *popcorn_fx, char *filename);

/// Install the latest available update.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to a `PopcornFX` instance.
void install_update(PopcornFX *popcorn_fx);

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

/// Logs a message sent over FFI using the Rust logger.
///
/// # Arguments
///
/// * `message` - A pointer to the null-terminated C string containing the log message to be logged.
/// * `level` - The log level of the message. Determines the verbosity of the message and how it will be formatted by the Rust logger.
void log(const char *target, const char *message, LogLevel level);

/// Retrieve the available subtitles for the given [MovieDetailsC].
///
/// It returns a reference to [SubtitleInfoSet], else a [ptr::null_mut] on failure.
/// <i>The returned reference should be managed by the caller.</i>
SubtitleInfoSet *movie_subtitles(PopcornFX *popcorn_fx, const MovieDetailsC *movie);

/// Create a new PopcornFX instance.
/// The caller will become responsible for managing the memory of the struct.
/// The instance can be safely deleted by using [dispose_popcorn_fx].
PopcornFX *new_popcorn_fx(const char **args, int32_t len);

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

/// Register a new callback listener for favorite events.
void register_favorites_event_callback(PopcornFX *popcorn_fx, void (*callback)(FavoriteEventC));

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

/// Register a new callback for all setting events.
void register_settings_callback(PopcornFX *popcorn_fx, ApplicationConfigCallbackC callback);

/// Register a new callback for subtitle events.
void register_subtitle_callback(PopcornFX *popcorn_fx, SubtitleCallbackC callback);

/// Register a new callback for the torrent stream.
void register_torrent_stream_callback(TorrentStreamC *stream, void (*callback)(TorrentStreamEventC));

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
MediaItemC *retrieve_favorite_details(PopcornFX *popcorn_fx, const char *imdb_id);

/// Retrieve the details of a given movie.
/// It will query the api for the given IMDB ID.
///
/// It returns the [MovieDetailsC] on success, else [ptr::null_mut].
MovieDetailsC *retrieve_movie_details(PopcornFX *popcorn_fx, const char *imdb_id);

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

/// Retrieve the details of a show based on the given IMDB ID.
/// The details contain all information about the show such as episodes and descriptions.
///
/// It returns the [ShowDetailsC] on success, else a [ptr::null_mut].
ShowDetailsC *retrieve_show_details(PopcornFX *popcorn_fx, const char *imdb_id);

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

/// Start a torrent stream for the given torrent.
TorrentStreamC *start_stream(PopcornFX *popcorn_fx, const TorrentWrapperC *torrent);

/// Stop the given torrent stream.
void stop_stream(PopcornFX *popcorn_fx, TorrentStreamC *stream);

/// Retrieve a special [SubtitleInfo::custom] type instance of the application.
SubtitleInfoC *subtitle_custom();

/// Retrieve a special [SubtitleInfo::none] type instance of the application.
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

/// Resolve the given torrent url into meta information of the torrent.
/// The url can be a magnet, http or file url to the torrent file.
void torrent_info(PopcornFX *popcorn_fx, const char *url);

/// Inform the FX core that a piece for the torrent has finished downloading.
void torrent_piece_finished(const TorrentWrapperC *torrent, uint32_t piece);

/// Inform the FX core that the state of the torrent has changed.
void torrent_state_changed(const TorrentWrapperC *torrent, TorrentState state);

/// Retrieve the current state of the stream.
/// Use [register_torrent_stream_callback] instead if the latest up-to-date information is required.
///
/// It returns the known [TorrentStreamState] at the time of invocation.
TorrentStreamState torrent_stream_state(TorrentStreamC *stream);

/// The torrent wrapper for moving data between rust and java.
/// This is a temp wrapper till the torrent component is replaced.
TorrentWrapperC *torrent_wrapper(TorrentC torrent);

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
