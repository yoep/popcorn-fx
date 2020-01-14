package com.github.yoep.popcorn.activities;

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
}
