package com.github.yoep.popcorn.ui.config;

import com.github.yoep.popcorn.Application;
import com.github.yoep.popcorn.PopcornFxManager;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitleDelegate;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleServiceImpl;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

import java.util.List;

@Slf4j
@Configuration
public class SubtitleConfig {
    @Bean
    public SubtitleService subtitleService(SettingsService settingsService) {
        return new SubtitleServiceImpl(settingsService, new SubtitleDelegate() {
            static final Object mutex = new Object();

            @Override
            public List<SubtitleInfo> defaultOptions() {
                return Application.INSTANCE.default_subtitle_options(PopcornFxManager.INSTANCE.fxInstance()).getSubtitles();
            }

            @Override
            public List<SubtitleInfo> subtitles(Movie movie) {
                var subtitles = Application.INSTANCE.movie_subtitles(PopcornFxManager.INSTANCE.fxInstance(), movie).getSubtitles();
                log.info("Retrieved subtitles: {}", subtitles);
                return subtitles;
            }

            @Override
            public List<SubtitleInfo> subtitles(Show show, Episode episode) {
                return Application.INSTANCE.episode_subtitles(PopcornFxManager.INSTANCE.fxInstance(), show, episode).getSubtitles();
            }

            @Override
            public List<SubtitleInfo> subtitles(String filename) {
                return Application.INSTANCE.filename_subtitles(PopcornFxManager.INSTANCE.fxInstance(), filename).getSubtitles();
            }

            @Override
            public SubtitleInfo getDefaultOrInterfaceLanguage(List<SubtitleInfo> subtitles) {
                var count = subtitles.size();
                var array = (SubtitleInfo[]) new SubtitleInfo().toArray(count);

                for (int i = 0; i < count; i++) {
                    var subtitle = subtitles.get(i);
                    array[i].imdbId = subtitle.imdbId;
                    array[i].language = subtitle.language;
                    array[i].infoPointer = subtitle.infoPointer;
                }

                return Application.INSTANCE.select_or_default_subtitle(PopcornFxManager.INSTANCE.fxInstance(), array, count);
            }

            @Override
            public Subtitle download(SubtitleInfo subtitle, SubtitleMatcher matcher) {
                synchronized (mutex) {
                    log.info("Sending matcher {}", matcher);
                    return Application.INSTANCE.download_subtitle(PopcornFxManager.INSTANCE.fxInstance(), subtitle, matcher);
                }
            }

            @Override
            public Subtitle parse(String filePath) {
                return Application.INSTANCE.parse_subtitle(PopcornFxManager.INSTANCE.fxInstance(), filePath);
            }

            @Override
            public String convert(Subtitle subtitle, SubtitleType type) {
                return Application.INSTANCE.subtitle_to_raw(PopcornFxManager.INSTANCE.fxInstance(), subtitle, type);
            }
        });
    }
}
