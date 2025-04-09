package com.github.yoep.popcorn.backend.lib;

import com.google.protobuf.MessageLite;

/**
 * A callback subscription for a certain message type.
 *
 * @param <T> The resulting type of the message that is being received.
 */
public interface FxCallback<T extends MessageLite> {
    void callback(T message);
}
