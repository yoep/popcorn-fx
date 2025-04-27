package com.github.yoep.popcorn.backend.updater;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.UpdateEvent;

public interface UpdateEventListener {
    void onStateChanged(UpdateEvent.StateChanged event);

    void onDownloadProgress(UpdateEvent.DownloadProgress event);
}
