package com.github.yoep.popcorn.backend.stream;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Stream;
import lombok.ToString;

import java.util.Objects;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedQueue;

@ToString
public class StreamServer implements IStreamServer {
    private final FxChannel fxChannel;

    private final Queue<StreamListenerHolder> listeners = new ConcurrentLinkedQueue<>();

    public StreamServer(FxChannel fxChannel) {
        Objects.requireNonNull(fxChannel, "fxChannel cannot be null");
        this.fxChannel = fxChannel;
        init();
    }

    @Override
    public void addListener(String filename, StreamListener listener) {
        this.listeners.add(new StreamListenerHolder(filename, listener));
    }

    @Override
    public void removeListener(String filename, StreamListener listener) {
        this.listeners.removeIf(holder -> listener == holder.listener());
    }

    private void init() {
        fxChannel.subscribe(FxChannel.typeFrom(Stream.StreamEvent.class), Stream.StreamEvent.parser(), this::onEvent);
    }

    private void onEvent(Stream.StreamEvent event) {
        var eventType = event.getType();
        listeners.forEach(holder -> {
            if (holder.filename().equals(event.getFilename())) {
                switch (eventType) {
                    case STATE_CHANGED -> holder.listener().onStateChanged(event.getState());
                    case STATS_CHANGED -> holder.listener().onStatsChanged(event.getStats());
                }
            }
        });
    }

    private record StreamListenerHolder(String filename, StreamListener listener) {

    }
}
