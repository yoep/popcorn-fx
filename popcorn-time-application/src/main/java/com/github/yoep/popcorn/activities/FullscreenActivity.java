package com.github.yoep.popcorn.activities;

public interface FullscreenActivity extends Activity {
    /**
     * Indicates with either the stage has entered fullscreen on the change.
     *
     * @return Returns true if the stage is in fullscreen, else false.
     */
    boolean isFullscreen();
}
