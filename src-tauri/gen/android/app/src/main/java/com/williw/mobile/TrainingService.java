package com.williw.mobile;

import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.app.Service;
import android.content.Context;
import android.content.Intent;
import android.os.Build;
import android.os.IBinder;
import android.util.Log;

import androidx.core.app.NotificationCompat;

/**
 * 训练后台服务 - 对应桌面版的后台训练功能
 */
public class TrainingService extends Service {
    private static final String TAG = "TrainingService";
    private static final String CHANNEL_ID = "williw_training_channel";
    private static final int NOTIFICATION_ID = 1;
    
    @Override
    public void onCreate() {
        super.onCreate();
        createNotificationChannel();
        Log.i(TAG, "训练服务已创建");
    }
    
    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        Log.i(TAG, "训练服务启动");
        
        // 启动前台服务通知
        startForeground(NOTIFICATION_ID, createNotification());
        
        // 这里可以启动实际的训练逻辑
        // 在真实实现中，这里会调用Rust库进行训练
        
        return START_STICKY; // 服务重启
    }
    
    @Override
    public IBinder onBind(Intent intent) {
        return null; // 不绑定
    }
    
    @Override
    public void onDestroy() {
        super.onDestroy();
        Log.i(TAG, "训练服务已销毁");
    }
    
    /**
     * 创建通知渠道（Android 8.0+）
     */
    private void createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            NotificationChannel channel = new NotificationChannel(
                CHANNEL_ID,
                "Williw训练",
                NotificationManager.IMPORTANCE_DEFAULT
            );
            channel.setDescription("去中心化训练后台服务");
            
            NotificationManager manager = getSystemService(NotificationManager.class);
            manager.createNotificationChannel(channel);
        }
    }
    
    /**
     * 创建前台服务通知
     */
    private Notification createNotification() {
        Intent notificationIntent = new Intent(this, MainActivity.class);
        PendingIntent pendingIntent = PendingIntent.getActivity(
            this, 0, notificationIntent, 
            PendingIntent.FLAG_UPDATE_CURRENT | PendingIntent.FLAG_IMMUTABLE
        );
        
        NotificationCompat.Builder builder = new NotificationCompat.Builder(this, CHANNEL_ID)
                .setContentTitle("Williw训练")
                .setContentText("去中心化训练正在后台运行")
                .setSmallIcon(android.R.drawable.ic_dialog_info)
                .setContentIntent(pendingIntent)
                .setPriority(NotificationCompat.PRIORITY_DEFAULT);
        
        return builder.build();
    }
}
