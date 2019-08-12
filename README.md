Queue-length-based Generic Congestion Control
===========================================

Queue-length-based Generic Congestion Control is a window-based congestion control algorithm, which adjust its congestion window based on the feedback from the SDCCP controller.

This repo is developed based on [Generic Congestion Avoidance CC of ccp-project](https://github.com/ccp-project/generic-cong-avoid).


## Notes

- In order to use this algorithm for congestion control, you also need to install a CCP datapath.
If you see errors about not being able to install a datapath program, it means that you have
either not installed a datapath, or the IPC mechanism between the algorithm and datapath is not
configured properly.