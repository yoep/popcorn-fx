package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.media.providers.Episode;
import com.github.yoep.popcorn.backend.media.providers.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.ShowDetails;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitlePreference;

import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

public interface SubtitleService {
    /**
     * Verify if the subtitle is disabled by the user.
     *
     * @return Returns true when disabled by the user, else false.
     */
    boolean isDisabled();

    /**
     * Retrieve the none/disabled subtitle info instance.
     */
    SubtitleInfo none();

    /**
     * Retrieve the custom subtitle info type.
     */
    SubtitleInfo custom();

    /**
     * Retrieve the available subtitles for the given media.
     *
     * @param media The media to retrieve the subtitles of.
     * @return Returns the list of available subtitles for the media.
     */
    CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(MovieDetails media);

    /**
     * Retrieve the available subtitles for the given media.
     *
     * @param media   The media to retrieve the subtitles of.
     * @param episode The episode of the media to retrieve the subtitle of.
     * @return Returns the list of available subtitles for the media.
     */
    CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(ShowDetails media, Episode episode);

    /**
     * Retrieve the available subtitles for the given filename.
     * This is a best effort of finding subtitles for videos files which are directly played back through the UI.
     *
     * @param filename The filename to retrieve the subtitle for.
     * @return Returns the list of available subtitles for the given file.
     */
    CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(String filename);

    /**
     * Download and parse the SRT file for the given {@link SubtitleInfo}.
     *
     * @param subtitleInfo The subtitle info to download and parse.
     * @return Returns the subtitle for the given subtitle info.
     */
    CompletableFuture<Subtitle> downloadAndParse(SubtitleInfo subtitleInfo, SubtitleMatcher.ByValue matcher);

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

    /**
     * Get the subtitle preference of the user for the current media.
     *
     * @return Returns the subtitle preference of the user for the current media.
     */
    SubtitlePreference preference();

    /**
     * Update the preferred subtitle for the media playback.
     * Passing `null` will disable the subtitle for the next media playback item.
     *
     * @param subtitle The new subtitle info to prefer on the next playback, or null.
     * @deprecated Use {@link #updatePreferredLanguage(SubtitleLanguage)} instead.
     */
    @Deprecated
    void updateSubtitle(SubtitleInfo subtitle);

    /**
     * Update the preferred subtitle language for the media playback.
     *
     * @param language The new subtitle language to prefer on the next playback.
     */
    void updatePreferredLanguage(SubtitleLanguage language);

    /**
     * Register a new subtitle callback which will be invoked for new {@link SubtitleEvent}'s.
     */
    void register(SubtitleEventCallback callback);

    /**
     * Disable the subtitle track.
     */
    void disableSubtitle();

    /**
     * Reset the active subtitle track to idle state.
     */
    void reset();

    /**
     * Clean the subtitles directory of all subtitle files.
     */
    void cleanup();
}
