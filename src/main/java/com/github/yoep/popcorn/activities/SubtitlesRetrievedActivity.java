package com.github.yoep.popcorn.activities;

import com.github.yoep.popcorn.subtitle.models.SubtitleInfo;

import java.util.List;

public interface SubtitlesRetrievedActivity extends Activity {
    /**
     * Get the IMDB id for which the subtitles were retrieved.
     *
     * @return Returns the IMDB id.
     */
    String getImdbId();

    /**
     * Get the subtitles that were retrieved.
     *
     * @return Returns the subtitles that were retrieved.
     */
    List<SubtitleInfo> getSubtitles();
}
