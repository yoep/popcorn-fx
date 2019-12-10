package com.github.yoep.popcorn.media.video.time;

public interface TimeListener {
    /**
     * Invoked when the duration of the media player is changed.
     *
     * @param newLength The new duration length of the media (number of milliseconds).
     */
    void onLengthChanged(long newLength);

    /**
     * Invoked when the current playback time of the media player is changed.
     *
     * @param newTime The new time of the media (number of milliseconds).
     */
    void onTimeChanged(long newTime);
}
