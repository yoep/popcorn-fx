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

enum class SubtitleType : int32_t {
  Srt = 0,
  Vtt = 1,
};

template<typename T = void>
struct Box;

/// The [PopcornFX] application instance.
struct PopcornFX;

/// The subtitle info contains information about available subtitles for a certain [Media].
/// This info includes a specific language for the media ID as well as multiple available files which can be used for smart subtitle detection.
struct SubtitleInfo;

struct SubtitleInfoC {
  const char *imdb_id;
  SubtitleLanguage language;
  SubtitleInfo *subtitle_info;
};

struct VecSubtitleInfoC {
  SubtitleInfoC *subtitles;
  int32_t len;
  int32_t cap;
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
  int32_t cap;
};

struct SubtitleCueC {
  const char *id;
  uint64_t start_time;
  uint64_t end_time;
  SubtitleLineC *lines;
  int32_t number_of_lines;
  int32_t capacity;
};

struct SubtitleC {
  const char *file;
  SubtitleInfoC info;
  SubtitleCueC *cues;
  int32_t number_of_cues;
  int32_t cues_capacity;
};

struct SubtitleMatcherC {
  const char *name;
  int32_t quality;
};

struct RatingC {
  int32_t percentage;
  int32_t watching;
  int32_t votes;
  int32_t loved;
  int32_t hated;
};

struct ImagesC {
  const char *poster;
  const char *fanart;
  const char *banner;
};

struct ShowC {
  const char *id;
  const char *tvdb_id;
  const char *title;
  const char *imdb_id;
  const char *year;
  int32_t runtime;
  RatingC *rating;
  ImagesC images;
  const char *synopsis;
};

struct EpisodeC {
  const char *id;
  const char *title;
  const char *imdb_id;
  const char *year;
  int32_t runtime;
  RatingC *rating;
  ImagesC images;
  const char *synopsis;
  int32_t season;
  int32_t episode;
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
};

struct TorrentQualityC {
  const char *quality;
  TorrentInfoC torrent;
};

struct TorrentEntryC {
  const char *language;
  TorrentQualityC *qualities;
  int32_t len;
  int32_t cap;
};

struct MovieC {
  const char *id;
  const char *title;
  const char *imdb_id;
  const char *year;
  int32_t runtime;
  RatingC *rating;
  ImagesC images;
  const char *synopsis;
  const char *trailer;
  TorrentEntryC *torrents;
  int32_t torrents_len;
  int32_t torrents_cap;
};

struct PlatformInfoC {
  /// The platform type
  PlatformType platform_type;
  /// The cpu architecture of the platform
  const char *arch;
};

struct VecMovieC {
  MovieC *movies;
  int32_t len;
  int32_t cap;
};

struct GenreC {
  const char *key;
  const char *text;
};

struct SortByC {
  const char *key;
  const char *text;
};


extern "C" {

/// Retrieve the default options available for the subtitles.
VecSubtitleInfoC *default_subtitle_options(PopcornFX *popcorn_fx);

/// Disable the screensaver on the current platform
void disable_screensaver(PopcornFX *popcorn_fx);

/// Delete the PopcornFX instance in a safe way.
void dispose_popcorn_fx(Box<PopcornFX> popcorn_fx);

/// Download and parse the given subtitle info.
///
/// It returns the [SubtitleC] reference on success, else [ptr::null_mut].
SubtitleC *download_subtitle(PopcornFX *popcorn_fx, const SubtitleInfoC *subtitle, const SubtitleMatcherC *matcher);

/// Enable the screensaver on the current platform
void enable_screensaver(PopcornFX *popcorn_fx);

/// Retrieve the given subtitles for the given episode
VecSubtitleInfoC *episode_subtitles(PopcornFX *popcorn_fx, const ShowC *show, const EpisodeC *episode);

/// Retrieve the available subtitles for the given filename
VecSubtitleInfoC *filename_subtitles(PopcornFX *popcorn_fx, char *filename);

/// Retrieve the available subtitles for the given [MovieC].
///
/// It returns a reference to [VecSubtitleInfoC], else a [ptr::null_mut] on failure.
/// <i>The returned reference should be managed by the caller.</i>
VecSubtitleInfoC *movie_subtitles(PopcornFX *popcorn_fx, const MovieC *movie);

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

/// Reset all available api stats for the movie api.
/// This will make all disabled api's available again.
void reset_movie_apis(PopcornFX *popcorn_fx);

/// Retrieve the available movies for the given criteria.
///
/// It returns the [VecMovieC] reference on success, else [ptr::null_mut].
VecMovieC *retrieve_available_movies(PopcornFX *popcorn_fx, const GenreC *genre, const SortByC *sort_by, const char *keywords, uint32_t page);

/// Retrieve the details of a given movie.
/// It will query the api for the given IMDB ID.
///
/// It returns the [MovieC] on success, else [ptr::null_mut].
MovieC *retrieve_movie_details(PopcornFX *popcorn_fx, const char *imdb_id);

/// Select a default subtitle language based on the settings or user interface language.
SubtitleInfoC *select_or_default_subtitle(PopcornFX *popcorn_fx, const SubtitleInfoC *subtitles_ptr, size_t len);

const char *subtitle_to_raw(PopcornFX *popcorn_fx, const SubtitleC *subtitle, const SubtitleType *output_type);

} // extern "C"
