#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>


/// The platform type
enum class PlatformType : int32_t {
  /// The windows platform
  Windows = 0,
  /// The macos platform
  MacOs = 1,
  /// The linux platform
  Linux = 2,
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

template<typename T = void>
struct Arc;

template<typename T = void>
struct Box;

/// The [PopcornFX] application instance.
struct PopcornFX;

/// The C compatible struct for [TorrentStream].
struct TorrentStreamC;

struct TorrentWrapper;

struct RatingC {
  uint16_t percentage;
  uint32_t watching;
  uint32_t votes;
  uint32_t loved;
  uint32_t hated;
};

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

struct EpisodeC {
  int32_t season;
  int32_t episode;
  int64_t first_aired;
  const char *title;
  const char *synopsis;
  const char *tvdb_id;
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

struct MediaItemC {
  MovieOverviewC *movie_overview;
  MovieDetailsC *movie_details;
  ShowOverviewC *show_overview;
  ShowDetailsC *show_details;
  EpisodeC *episode;
};

struct SubtitleFileC {
  int32_t file_id;
  const char *name;
  const char *url;
  float score;
  int32_t downloads;
  int32_t *quality;
};

struct SubtitleInfoC {
  const char *imdb_id;
  SubtitleLanguage language;
  SubtitleFileC *files;
  int32_t len;
};

struct VecSubtitleInfoC {
  SubtitleInfoC *subtitles;
  int32_t len;
  int32_t cap;
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

/// The subtitle matcher C compatible struct.
/// It contains the information which should be matched when selecting a subtitle file to load.
struct SubtitleMatcherC {
  /// The nullable name of the media item.
  const char *name;
  /// The nullable quality of the media item.
  /// This can be represented as `720p` or `720`.
  const char *quality;
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

struct SubtitleC {
  const char *file;
  SubtitleInfoC info;
  SubtitleCueC *cues;
  int32_t number_of_cues;
};

struct PlatformInfoC {
  /// The platform type
  PlatformType platform_type;
  /// The cpu architecture of the platform
  const char *arch;
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

/// Structure holding the values of a string array.
struct StringArray {
  const char **values;
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

/// The callback for prioritizing pieces.
using PrioritizePiecesCallbackC = void(*)(int32_t, uint32_t*);

/// The callback for update the torrent mode to sequential.
using SequentialModeCallbackC = void(*)();

/// The C compatible abi struct for a [Torrent].
/// This currently uses callbacks as it's a wrapper around a torrent implementation provided through C.
struct TorrentC {
  /// The filepath to the torrent file
  const char *filepath;
  HasByteCallbackC has_byte_callback;
  HasPieceCallbackC has_piece_callback;
  TotalPiecesCallbackC total_pieces;
  PrioritizePiecesCallbackC prioritize_pieces;
  SequentialModeCallbackC sequential_mode;
};


extern "C" {

/// Add the media item to the favorites.
/// Duplicate favorite media items are ignored.
void add_to_favorites(PopcornFX *popcorn_fx, const MediaItemC *favorite);

/// Add the given media item to the watched list.
void add_to_watched(PopcornFX *popcorn_fx, const MediaItemC *watchable);

/// Retrieve the default options available for the subtitles.
VecSubtitleInfoC *default_subtitle_options(PopcornFX *popcorn_fx);

/// Disable the screensaver on the current platform
void disable_screensaver(PopcornFX *popcorn_fx);

/// Dispose the given media item from memory.
void dispose_media_item(Box<MediaItemC> media);

/// Dispose all given media items from memory.
void dispose_media_items(Box<MediaSetC> media);

/// Delete the PopcornFX instance in a safe way.
void dispose_popcorn_fx(Box<PopcornFX> popcorn_fx);

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

/// Enable the screensaver on the current platform
void enable_screensaver(PopcornFX *popcorn_fx);

/// Retrieve the given subtitles for the given episode
VecSubtitleInfoC *episode_subtitles(PopcornFX *popcorn_fx, const ShowDetailsC *show, const EpisodeC *episode);

/// Retrieve the available subtitles for the given filename
VecSubtitleInfoC *filename_subtitles(PopcornFX *popcorn_fx, char *filename);

/// Verify if the given media item is liked/favorite of the user.
/// It will use the first non [ptr::null_mut] field from the [MediaItemC] struct.
///
/// It will return false if all fields in the [MediaItemC] are [ptr::null_mut].
bool is_media_liked(PopcornFX *popcorn_fx, const MediaItemC *favorite);

/// Verify if the given media item is watched by the user.
///
/// It returns true when the item is watched, else false.
bool is_media_watched(PopcornFX *popcorn_fx, const MediaItemC *watchable);

/// Retrieve the available subtitles for the given [MovieDetailsC].
///
/// It returns a reference to [VecSubtitleInfoC], else a [ptr::null_mut] on failure.
/// <i>The returned reference should be managed by the caller.</i>
VecSubtitleInfoC *movie_subtitles(PopcornFX *popcorn_fx, const MovieDetailsC *movie);

/// Create a new PopcornFX instance.
/// The caller will become responsible for managing the memory of the struct.
/// The instance can be safely deleted by using [delete_popcorn_fx].
PopcornFX *new_popcorn_fx();

/// Parse the given subtitle file.
///
/// It returns the parsed subtitle on success, else null.
SubtitleC *parse_subtitle(PopcornFX *popcorn_fx, const char *file_path);

/// Retrieve the platform information
PlatformInfoC *platform_info(PopcornFX *popcorn_fx);

/// Register a new callback listener for favorite events.
void register_favorites_event_callback(PopcornFX *popcorn_fx, void (*callback)(FavoriteEventC));

/// Register a new callback for the torrent stream.
void register_torrent_stream_callback(TorrentStreamC *stream, void (*callback)(TorrentStreamEventC));

/// Register a new callback listener for watched events.
void register_watched_event_callback(PopcornFX *popcorn_fx, void (*callback)(WatchedEventC));

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
MediaSetC *retrieve_available_movies(PopcornFX *popcorn_fx, const GenreC *genre, const SortByC *sort_by, const char *keywords, uint32_t page);

/// Retrieve the available [ShowOverviewC] items for the given criteria.
///
/// It returns an array of [ShowOverviewC] items on success, else a [ptr::null_mut].
MediaSetC *retrieve_available_shows(PopcornFX *popcorn_fx, const GenreC *genre, const SortByC *sort_by, const char *keywords, uint32_t page);

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

/// Update the preferred subtitle for the [Media] item playback.
/// This action will reset any custom configured subtitle files.
void update_subtitle(PopcornFX *popcorn_fx, const SubtitleInfoC *subtitle);

/// Update the preferred subtitle to a custom subtitle filepath.
/// This action will reset any preferred subtitle.
void update_subtitle_custom_file(PopcornFX *popcorn_fx, const char *custom_filepath);

} // extern "C"
