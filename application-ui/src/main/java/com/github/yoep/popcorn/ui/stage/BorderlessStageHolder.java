package com.github.yoep.popcorn.ui.stage;

import com.github.spring.boot.javafx.stage.BorderlessStageWrapper;
import lombok.Data;
import lombok.Setter;

import java.util.Optional;

@Data
public class BorderlessStageHolder {
    private static final BorderlessStageHolder holder = new BorderlessStageHolder();

    @Setter
    private BorderlessStageWrapper stageWrapper;

    private BorderlessStageHolder() {
    }

    public static Optional<BorderlessStageWrapper> getWrapper() {
        return Optional.ofNullable(holder.getStageWrapper());
    }

    public static void setWrapper(BorderlessStageWrapper wrapper) {
        holder.setStageWrapper(wrapper);
    }
}
