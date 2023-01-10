package com.github.yoep.popcorn.backend;

import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.media.providers.MovieSet;
import com.github.yoep.popcorn.backend.media.providers.ShowSet;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.platform.PlatformInfo;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfoSet;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import com.sun.jna.Library;
import com.sun.jna.Native;

/**
 * The Popcorn FX native library interface.
 * Use the {@link FxLib#INSTANCE} to communicate with the loaded library.
 * <p>
 * <i>Example:</i>
 * <p>
 * <code>
 * var subtitles = FxLib.INSTANCE.movie_subtitles(PopcornFxInstance.INSTANCE.get(), movie);
 * </code>
 */
public interface FxLib extends Library {
    FxLib INSTANCE = Native.load("popcorn_fx", FxLib.class);

    PopcornFx new_popcorn_fx();

    PlatformInfo platform_info(PopcornFx instance);

    SubtitleInfoSet default_subtitle_options(PopcornFx instance);

    SubtitleInfoSet movie_subtitles(PopcornFx instance, Movie movie);

    SubtitleInfoSet episode_subtitles(PopcornFx instance, ShowDetails show, Episode episode);

    SubtitleInfoSet filename_subtitles(PopcornFx instance, String filename);

    SubtitleInfo select_or_default_subtitle(PopcornFx instance, SubtitleInfo[] subtitles, int len);

    Subtitle download_subtitle(PopcornFx instance, SubtitleInfo subtitle, SubtitleMatcher matcher);

    Subtitle parse_subtitle(PopcornFx instance, String filePath);

    String subtitle_to_raw(PopcornFx instance, Subtitle subtitle, int type);

    MovieSet retrieve_available_movies(PopcornFx instance, Genre genre, SortBy sort, String keywords, int page);

    Movie retrieve_movie_details(PopcornFx instance, String imdbId);

    void reset_movie_apis(PopcornFx instance);

    ShowSet retrieve_available_shows(PopcornFx instance, Genre genre, SortBy sort, String keywords, int page);

    ShowDetails retrieve_show_details(PopcornFx instance, String imdbId);

    void reset_show_apis(PopcornFx instance);

    void disable_screensaver(PopcornFx instance);

    void enable_screensaver(PopcornFx instance);

    void dispose_popcorn_fx(PopcornFx instance);
}