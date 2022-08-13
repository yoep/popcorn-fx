package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleCue;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;
import javafx.beans.property.ReadOnlyObjectProperty;
import org.springframework.scheduling.annotation.Async;

import java.io.File;
import java.io.InputStream;
import java.nio.charset.Charset;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

public interface SubtitleService {
    /**
     * Get the current subtitle of the video player.
     *
     * @return Returns the subtitle.
     */
    Optional<Subtitle> getActiveSubtitle();

    /**
     * Get the subtitle property.
     *
     * @return Returns the subtitle property.
     */
    ReadOnlyObjectProperty<Subtitle> activeSubtitleProperty();

    /**
     * Set the subtitle for the video player.
     *
     * @param activeSubtitle The subtitle for the video player.
     */
    void setActiveSubtitle(Subtitle activeSubtitle);

    /**
     * Retrieve the available subtitles for the given media.
     *
     * @param media The media to retrieve the subtitles of.
     * @return Returns the list of available subtitles for the media.
     */
    @Async
    CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(Movie media);

    /**
     * Retrieve the available subtitles for the given media.
     *
     * @param media   The media to retrieve the subtitles of.
     * @param episode The episode of the media to retrieve the subtitle of.
     * @return Returns the list of available subtitles for the media.
     */
    @Async
    CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(Show media, Episode episode);

    /**
     * Retrieve the available subtitles for the given filename.
     * This is a best effort of finding subtitles for videos files which are directly played back through the UI.
     *
     * @param filename The filename to retrieve the subtitle for.
     * @return Returns the list of available subtitles for the given file.
     */
    @Async
    CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(String filename);

    /**
     * Parse the given SRT file to a list of {@link SubtitleCue}'s.
     *
     * @param file     The SRT file to parse.
     * @param encoding The encoding of the SRT file.
     * @return Returns the parsed subtitle.
     */
    @Async
    CompletableFuture<Subtitle> parse(File file, Charset encoding);

    /**
     * Download and parse the SRT file for the given {@link SubtitleInfo}.
     *
     * @param subtitleInfo The subtitle info to download and parse.
     * @return Returns the subtitle for the given subtitle info.
     */
    @Async
    CompletableFuture<Subtitle> downloadAndParse(SubtitleInfo subtitleInfo, SubtitleMatcher matcher);

    /**
     * Convert the given subtitle to the format type.
     * It will output a string stream of the converted subtitle.
     *
     * @param subtitle The subtitle to convert.
     * @param type     The expected subtitle type.
     * @return Returns the converted subtitle output.
     */
    InputStream convert(Subtitle subtitle, SubtitleType type);

    /**
     * Get the subtitle that needs to be selected by default for the given subtitles list.
     * This is based on the subtitle settings and tries to find the user's preferred language if it exists or uses the interface language if not found.
     * If the user's preferred language doesn't exist in the list, it will use the interface language.
     * If both the user's preferred language and interface language don't exist, it returns the default {@link SubtitleInfo#none()} subtitle.
     *
     * @param subtitles The subtitle list to search for the preferred language.
     * @return Returns the subtitle that needs to be selected by default.
     */
    SubtitleInfo getDefaultOrInterfaceLanguage(List<SubtitleInfo> subtitles);
}
