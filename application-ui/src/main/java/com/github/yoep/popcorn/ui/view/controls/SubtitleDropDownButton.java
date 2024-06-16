package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import javafx.scene.image.Image;
import lombok.extern.slf4j.Slf4j;

import java.io.InputStream;
import java.util.Optional;

@Slf4j
public class SubtitleDropDownButton extends DropDownButton<SubtitleInfo> {
    public SubtitleDropDownButton() {
        super(createItemFactory());
    }

    private static Image resourceToImage(InputStream resource) {
        return new Image(resource, 16, 16, true, true);
    }

    private static DropDownButtonFactory<SubtitleInfo> createItemFactory() {
        return new DropDownButtonFactory<>() {
            @Override
            public String getId(SubtitleInfo item) {
                return Optional.ofNullable(item.imdbId())
                        .orElseGet(() -> item.language().getCode());
            }

            @Override
            public String displayName(SubtitleInfo item) {
                return item
                        .language()
                        .getNativeName();
            }

            @Override
            public Image graphicResource(SubtitleInfo item) {
                return resourceToImage(SubtitleDropDownButton.class.getResourceAsStream(item.getFlagResource()));
            }
        };
    }
}
