package com.github.yoep.popcorn;

import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfoSet;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;
import com.github.yoep.popcorn.platform.PlatformInfo;
import com.sun.jna.Library;
import com.sun.jna.Native;

public interface Application extends Library {
    Application INSTANCE = Native.load("popcorn_fx", Application.class);

    PopcornFx new_popcorn_fx();

    PlatformInfo.ByValue platform_info(PopcornFx instance);

    SubtitleInfoSet default_subtitle_options(PopcornFx instance);

    SubtitleInfoSet movie_subtitles(PopcornFx instance, Movie movie);

    SubtitleInfoSet episode_subtitles(PopcornFx instance, Show show, Episode episode);

    SubtitleInfoSet filename_subtitles(PopcornFx instance, String filename);

    SubtitleInfo select_or_default_subtitle(PopcornFx instance, SubtitleInfo[] subtitles, int len);

    Subtitle download_subtitle(PopcornFx instance, SubtitleInfo subtitle, SubtitleMatcher matcher);

    Subtitle parse_subtitle(PopcornFx instance, String filePath);
    
    String subtitle_to_raw(PopcornFx instance, Subtitle subtitle, SubtitleType type);

    void disable_screensaver(PopcornFx instance);

    void enable_screensaver(PopcornFx instance);

    void dispose_popcorn_fx(PopcornFx instance);
}