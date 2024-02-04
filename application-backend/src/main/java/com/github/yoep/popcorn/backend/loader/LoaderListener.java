package com.github.yoep.popcorn.backend.loader;

public interface LoaderListener {
    void onLoadingStarted(LoadingStartedEventC loadingStartedEvent);

    void onStateChanged(LoaderState newState);

    void onProgressChanged(LoadingProgress progress);
    
    void onError(LoadingErrorC error);
}
