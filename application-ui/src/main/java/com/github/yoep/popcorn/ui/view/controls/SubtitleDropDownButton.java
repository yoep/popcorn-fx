package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import javafx.scene.image.Image;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.Resource;

import java.io.IOException;

@Slf4j
public class SubtitleDropDownButton extends DropDownButton<SubtitleInfo> {
    public SubtitleDropDownButton() {
        super(createItemFactory());
    }

    private static Image resourceToImage(Resource resource) {
        try {
            return new Image(resource.getInputStream(), 16, 16, true, true);
        } catch (IOException e) {
            log.error("Failed to load subtitle graphic resource, {}", e.getMessage(), e);
            return null;
        }
    }

    private static DropDownButtonFactory<SubtitleInfo> createItemFactory() {
        return new DropDownButtonFactory<>() {
            @Override
            public String displayName(SubtitleInfo item) {
                return item
                        .getLanguage()
                        .getNativeName();
            }

            @Override
            public Image graphicResource(SubtitleInfo item) {
                return resourceToImage(item.getFlagResource());
            }
        };
    }
}
