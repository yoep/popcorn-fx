package com.github.yoep.popcorn.backend.lib;

public interface FxReplyCallback<T> {
    void callback(Integer sequenceId, T message);
}
