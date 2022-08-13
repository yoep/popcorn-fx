package com.github.yoep.popcorn.backend.adapters.torrent;

/**
 * Service managing the torrent session settings.
 */
public interface TorrentSettingsService {
    /**
     * Update the global limit on the number of connections opened by the torrent session.
     *
     * @param connectionsLimit The max number of torrent connections.
     * @throws TorrentException Is thrown when the setting could not be applied.
     */
    TorrentSettingsService connectionsLimit(int connectionsLimit);

    /**
     * Update the session-global limits of download rate limit, in bytes per second.
     * A value of 0 means unlimited.
     *
     * @param downloadRateLimit The download rate limit in bytes per second.
     */
    TorrentSettingsService downloadRateLimit(int downloadRateLimit);

    /**
     * Update the session-global limits of upload rate limit, in bytes per second.
     *
     * @param uploadRateLimit The upload rate limit in bytes per second.
     */
    TorrentSettingsService uploadRateLimit(int uploadRateLimit);
}
