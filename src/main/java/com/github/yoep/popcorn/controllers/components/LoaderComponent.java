package com.github.yoep.popcorn.controllers.components;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.torrent.TorrentStream;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;

@Slf4j
@Component
@RequiredArgsConstructor
public class LoaderComponent {
    private final ActivityManager activityManager;
    private final TorrentStream torrentStream;


}
