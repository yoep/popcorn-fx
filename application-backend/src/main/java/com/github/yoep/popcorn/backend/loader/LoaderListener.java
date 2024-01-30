package com.github.yoep.popcorn.backend.loader;

public interface LoaderListener {
    void onStateChanged(LoaderState newState);
}
