package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.sun.jna.Structure;
import lombok.Builder;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.HashMap;
import java.util.Optional;

public record TorrentQuality(HashMap<String, Media.TorrentInfo> torrents) {
}
