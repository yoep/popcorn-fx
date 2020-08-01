package com.github.yoep.popcorn.ui.activities;

public interface PlayVideoActivity extends Activity {
    /**
     * Get the url of the video to play.
     *
     * @return Returns the video url.
     */
    String getUrl();

    /**
     * Get the title of the video.
     *
     * @return Returns the title of the video.
     */
    String getTitle();

    /**
     * Check if the subtitles should be enabled for this video playback.
     * If true, the player will show the subtitle UI section, otherwise it will be hidden.
     *
     * @return Returns true if the subtitles should be enabled, else false.
     */
    boolean isSubtitlesEnabled();
}
