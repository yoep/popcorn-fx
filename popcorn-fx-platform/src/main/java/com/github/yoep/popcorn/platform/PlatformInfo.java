package com.github.yoep.popcorn.platform;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformType;
import com.sun.jna.DefaultTypeMapper;
import com.sun.jna.Structure;
import com.sun.jna.ToNativeContext;
import com.sun.jna.platform.EnumConverter;
import lombok.Getter;

import java.io.Closeable;
import java.util.Optional;

@Getter
@Structure.FieldOrder({"type", "arch"})
public class PlatformInfo extends Structure implements com.github.yoep.popcorn.backend.adapters.platform.PlatformInfo, Closeable {
    public static class ByValue extends PlatformInfo implements Structure.ByValue {

    }

    public PlatformInfo() {
        super(new DefaultTypeMapper() {{
            addTypeConverter(PlatformType.class, new EnumConverter<>(PlatformType.class) {
                @Override
                public Integer toNative(Object input, ToNativeContext context) {
                    return Optional.ofNullable(input)
                            .map(e -> super.toNative(input, context))
                            .orElse(-1);
                }
            });
        }});
    }

    public PlatformType type;

    public String arch;

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
