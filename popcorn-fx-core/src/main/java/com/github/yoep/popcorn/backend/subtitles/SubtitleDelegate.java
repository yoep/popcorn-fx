package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;

import java.util.List;

public interface SubtitleDelegate {
    List<SubtitleInfo> defaultOptions();

    List<SubtitleInfo> subtitles(Movie movie);

    List<SubtitleInfo> subtitles(Show show, Episode episode);

    List<SubtitleInfo> subtitles(String filename);

    SubtitleInfo getDefaultOrInterfaceLanguage(List<SubtitleInfo> subtitles);

    Subtitle download(SubtitleInfo subtitle, SubtitleMatcher matcher);

    Subtitle parse(String filePath);

    String convert(Subtitle subtitle, SubtitleType type);
}
