package com.github.yoep.popcorn.ui.media.resume.models;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;

import java.util.ArrayList;
import java.util.List;

@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class AutoResume {
    @Builder.Default
    private List<VideoTimestamp> videoTimestamps = new ArrayList<>();
}
