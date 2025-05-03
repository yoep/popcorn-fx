package com.github.yoep.popcorn.ui.info;

import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.adapters.video.listeners.VideoListener;
import com.github.yoep.popcorn.backend.adapters.video.state.VideoState;
import com.github.yoep.popcorn.backend.info.ComponentInfo;
import com.github.yoep.popcorn.backend.info.ComponentState;
import com.github.yoep.popcorn.backend.info.SimpleComponentDetails;
import com.github.yoep.popcorn.ui.IoC;
import lombok.extern.slf4j.Slf4j;

import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

@Slf4j
public class VideoInfoService extends AbstractInfoService {
    private final IoC ioc;

    public VideoInfoService(IoC ioc) {
        Objects.requireNonNull(ioc, "ioc cannot be null");
        this.ioc = ioc;
    }

    @Override
    public List<ComponentInfo> getComponentDetails() {
        var videoPlaybacks = ioc.getInstances(VideoPlayback.class);
        updateComponents(videoPlaybacks.stream()
                .map(this::createComponentDetail)
                .collect(Collectors.toList()));
        return super.getComponentDetails();
    }

    private ComponentInfo createComponentDetail(VideoPlayback videoPlayback) {
        var componentDetails = SimpleComponentDetails.builder()
                .name(videoPlayback.getName())
                .description(videoPlayback.getDescription())
                .state(mapVideoStateToComponentState(videoPlayback.getVideoState()))
                .build();

        videoPlayback.addListener(new VideoListener() {
            @Override
            public void onDurationChanged(long newDuration) {
                // no-op
            }

            @Override
            public void onTimeChanged(long newTime) {
                // no-op
            }

            @Override
            public void onStateChanged(VideoState newState) {
                componentDetails.setState(mapVideoStateToComponentState(newState));
            }

            @Override
            public void onVolumeChanged(int volume) {
                // no-op
            }
        });

        return componentDetails;
    }

    private static ComponentState mapVideoStateToComponentState(VideoState videoState) {
        switch (videoState) {
            case UNKNOWN:
                return ComponentState.UNKNOWN;
            case ERROR:
                return ComponentState.ERROR;
            default:
                return ComponentState.READY;
        }
    }
}
