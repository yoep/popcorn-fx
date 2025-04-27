package com.github.yoep.popcorn.backend.loader;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.LoaderEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Loading;

public interface LoaderListener {
    void onLoadingStarted(LoaderEvent.LoadingStarted loadingStartedEvent);

    void onStateChanged(Loading.State newState);

    void onProgressChanged(Loading.Progress progress);
    
    void onError(Loading.Error error);
}
