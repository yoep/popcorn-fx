package com.github.yoep.popcorn.backend.playlists;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Playlist;
import com.github.yoep.popcorn.backend.media.Episode;
import com.github.yoep.popcorn.backend.media.MovieDetails;
import com.github.yoep.popcorn.backend.media.ShowDetails;
import com.github.yoep.popcorn.backend.services.ListenerService;

public interface PlaylistManager extends ListenerService<PlaylistManagerListener> {
    /**
     * Start the playback of the given playlist.
     *
     * @param playlist The playlist to play
     */
    void play(Playlist playlist);

    /**
     * Start the playback for the given movie media item with the given quality.
     *
     * @param movie   The movie media item
     * @param quality The quality of the item to play
     */
    void play(MovieDetails movie, String quality);

    /**
     * Start the playback for the given show media item with the given episode and quality.
     *
     * @param show    The show media item
     * @param episode The episode media item
     * @param quality The quality of the item to play
     */
    void play(ShowDetails show, Episode episode, String quality);

    /**
     * Start the playback of the next item in the playlist if available.
     * Has no effect when the playlist is empty.
     */
    void playNext();

    /**
     * Stop the playback of the current playlist.
     * This will clear the active playlist.
     */
    void stop();

    /**
     * Retrieve the currently active playlist.
     * If there is no playlist being played, this will return an empty playlist.
     *
     * @return Returns the currently active playlist.
     */
    Playlist playlist();
}
