package com.github.yoep.popcorn.torrent.listeners;

import com.frostwire.jlibtorrent.AlertListener;
import com.frostwire.jlibtorrent.DhtRoutingBucket;
import com.frostwire.jlibtorrent.alerts.Alert;
import com.frostwire.jlibtorrent.alerts.AlertType;
import com.frostwire.jlibtorrent.alerts.DhtStatsAlert;

import java.util.ArrayList;
import java.util.Iterator;

public abstract class DHTStatsAlertListener implements AlertListener {
    public DHTStatsAlertListener() {
    }

    public int[] types() {
        return new int[]{AlertType.DHT_STATS.swig()};
    }

    public void alert(Alert<?> alert) {
        if (alert instanceof DhtStatsAlert) {
            DhtStatsAlert dhtAlert = (DhtStatsAlert)alert;
            this.stats(this.countTotalDHTNodes(dhtAlert));
        }

    }

    public abstract void stats(int totalDhtNodes);

    private int countTotalDHTNodes(DhtStatsAlert alert) {
        ArrayList<DhtRoutingBucket> routingTable = alert.routingTable();
        int totalNodes = 0;
        DhtRoutingBucket bucket;
        if (routingTable != null) {
            for(Iterator var4 = routingTable.iterator(); var4.hasNext(); totalNodes += bucket.numNodes()) {
                bucket = (DhtRoutingBucket)var4.next();
            }
        }

        return totalNodes;
    }
}