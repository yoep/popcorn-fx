package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle.Language;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.SubtitleEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.SubtitlePreference;
import com.github.yoep.popcorn.backend.media.Episode;
import com.github.yoep.popcorn.backend.media.MovieDetails;
import com.github.yoep.popcorn.backend.media.ShowDetails;

import java.util.List;
import java.util.concurrent.CompletableFuture;

public interface ISubtitleService {

    /**
     * Get the default subtitles of the application.
     *
     * @return Returns the list of available subtitles.
     */
    CompletableFuture<List<ISubtitleInfo>> defaultSubtitles();

    /**
     * Retrieve the available subtitles for the given media.
     *
     * @param media The media to retrieve the subtitles of.
     * @return Returns the list of available subtitles for the media.
     */
    CompletableFuture<List<ISubtitleInfo>> retrieveSubtitles(MovieDetails media);

    /**
     * Retrieve the available subtitles for the given media.
     *
     * @param media   The media to retrieve the subtitles of.
     * @param episode The episode of the media to retrieve the subtitle of.
     * @return Returns the list of available subtitles for the media.
     */
    CompletableFuture<List<ISubtitleInfo>> retrieveSubtitles(ShowDetails media, Episode episode);

    /**
     * Retrieve the available subtitles for the given filename.
     * This is a best effort of finding subtitles for videos files which are directly played back through the UI.
     *
     * @param filename The filename to retrieve the subtitle for.
     * @return Returns the list of available subtitles for the given file.
     */
    CompletableFuture<List<ISubtitleInfo>> retrieveSubtitles(String filename);

    /**
     * Download and parse the SRT file for the given {@link Subtitle.Info}.
     *
     * @param subtitleInfo The subtitle info to download and parse.
     * @return Returns the subtitle for the given subtitle info.
     */
    CompletableFuture<ISubtitle> downloadAndParse(ISubtitleInfo subtitleInfo, Subtitle.Matcher matcher);

    /**
     * Get the subtitle that needs to be selected by default for the given subtitles list.
     * This is based on the subtitle settings and tries to find the user's preferred language if it exists or uses the interface language if not found.
     * If the user's preferred language doesn't exist in the list, it will use the interface language.
     * If both the user's preferred language and interface language don't exist, it returns the default {@link Language#NONE} subtitle.
     *
     * @param subtitles The subtitle list to search for the preferred language.
     * @return Returns the subtitle that needs to be selected by default.
     */
    CompletableFuture<ISubtitleInfo> getDefaultOrInterfaceLanguage(List<ISubtitleInfo> subtitles);

    /**
     * Get the subtitle preference of the user for the current media.
     *
     * @return Returns the subtitle preference of the user for the current media.
     */
    CompletableFuture<SubtitlePreference> preference();

    /**
     * Update the preferred subtitle language for the media playback.
     *
     * @param language The new subtitle language to prefer on the next playback.
     */
    void updatePreferredLanguage(Language language);

    /**
     * Register a new subtitle callback which will be invoked for new {@link SubtitleEvent}'s.
     */
    void register(FxCallback<SubtitleEvent> callback);

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
