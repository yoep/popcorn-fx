package com.github.yoep.popcorn.ui.info;

import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.adapters.video.listeners.VideoListener;
import com.github.yoep.popcorn.backend.adapters.video.state.VideoState;
import com.github.yoep.popcorn.backend.info.ComponentInfo;
import com.github.yoep.popcorn.backend.info.ComponentState;
import com.github.yoep.popcorn.backend.info.SimpleComponentDetails;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.util.List;
import java.util.stream.Collectors;

@Slf4j
@Service
@RequiredArgsConstructor
public class VideoInfoService extends AbstractInfoService {
    private final List<VideoPlayback> videoPlaybacks;

    @PostConstruct
    void init() {
        updateComponents(videoPlaybacks.stream()
                .map(this::createComponentDetail)
                .collect(Collectors.toList()));
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
