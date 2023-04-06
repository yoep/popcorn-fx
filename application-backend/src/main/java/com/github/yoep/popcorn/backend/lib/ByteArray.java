package com.github.yoep.popcorn.backend.lib;

import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.Getter;

import java.io.Closeable;
import java.util.Optional;

@Getter
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"values", "len"})
public class ByteArray extends Structure implements Closeable {
    public Pointer values;
    public int len;

    private byte[] cache;

    public byte[] getBytes() {
        return cache;
    }

    @Override
    public void read() {
        super.read();
        cache = Optional.ofNullable(values)
                .map(e -> e.getByteArray(0, len))
                .orElse(new byte[0]);
    }

    @Override
    public void close() {
        setAutoSynch(false);
        FxLibInstance.INSTANCE.get().dispose_byte_array(this);
    }
}
